use std::collections::HashMap;
use std::{env, vec, fs, sync::Arc};
use std::{time::Duration, collections::VecDeque};

use rusqlite::{Result, params};
use songbird::{SerenityInit, Songbird};
use tokio_rusqlite::Connection as Connection;
use tokio::{
    sync::RwLock,
    signal::ctrl_c,
};

use serenity::{
    async_trait,
    FutureExt,
    client::{Client, Context, EventHandler},
    prelude::*,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework, CommandError, Args
    },
    model::{
        prelude::{Message, Reaction, ReactionType, Ready},
        application::component::{SelectMenu, ComponentType, SelectMenuOption},
        application::interaction::InteractionResponseType,
        id::GuildId,
    },
};

use songbird::SongbirdKey;

use env_logger::Env;

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
    
    let shard_manager = client.shard_manager.clone();
    let client_future = client.start_autosharded().fuse();

    tokio::pin!(client_future);

    let ctrl_c = ctrl_c().fuse();
    tokio::pin!(ctrl_c);

    tokio::select! {
        client_result = client_future => {
            if let Err(why) = client_result {
                println!("An error occurred while running the client: {:?}", why);
            }
        },
        _ = ctrl_c => {
            println!("Ctrl-C received, shutting down");
            shard_manager.lock().await.shutdown_all().await;
        }
    };

    Ok(())
}