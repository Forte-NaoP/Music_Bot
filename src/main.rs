use std::collections::HashMap;
use std::{env, vec, fs, sync::Arc};
use std::{time::Duration, collections::VecDeque};

use rusqlite::{Result, params};
use songbird::SerenityInit;
use tokio_rusqlite::Connection as Connection;
use tokio::sync::{RwLock};

use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::prelude::*;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework, CommandError, Args
};
use serenity::model::{
    prelude::{Message, Reaction, ReactionType, Ready},
    application::component::{SelectMenu, ComponentType, SelectMenuOption},
    application::interaction::InteractionResponseType,
    id::GuildId,
};

use songbird::input::Input;

use env_logger::Env;
use ctrlc;

mod command_handler;
mod event_handler;
mod database_handler;
mod connection_handler;
mod utils;

use crate::utils::guild_queue::GuildQueue;

struct DBContainer;
impl TypeMapKey for DBContainer {
    type Value = Connection;
}

struct GuildQueueContainer;
impl TypeMapKey for GuildQueueContainer {
    type Value = HashMap<GuildId, Arc<RwLock<GuildQueue>>>;
}

#[tokio::main]
async fn main() -> Result<()> {
    let log_env = Env::default()
        .filter_or("RUST_LOG", "error");

    env_logger::init_from_env(log_env);

    let conn = Connection::open("music.db").await?;
    database_handler::initialize(&conn).await?;

    let framework = StandardFramework::new();

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGES
    ;

    let mut client = Client::builder(
        token, 
        intents
    )
    .event_handler(event_handler::event_handler::DiscordEventHandler)
    .register_songbird()
    .framework(framework)
    .await
    .expect("Error creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<DBContainer>(conn);
        data.insert::<GuildQueueContainer>(HashMap::default());
    }
    
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}