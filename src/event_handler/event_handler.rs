use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        prelude::{Message, Ready},
        application::{
            interaction::Interaction,
            command::Command,
        },
    },
};

use crate::{
    command_handler::{
        command_handler::*,
        commands::*,
    },
    utils::guild_queue::{self},
    GuildQueueContainer
};

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());

        guild_queue::initialize(&ctx).await;

        let commands = Command::get_global_application_commands(&ctx.http).await.unwrap();

        match commands.iter().find(|command| {
            "launch" == command.name.as_str()
        }) {
            Some(command) => command,
            None => &Command::create_global_application_command(&ctx.http, |command| {
                launch::register(command)
            }).await.unwrap()
        };
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => match command.data.name.as_str() {
                "launch" => launch::run(&ctx, command).await,
                _ => execute_command(&ctx, command).await,
            }
            Interaction::MessageComponent(component) => {},
            _ => {},
        };
    }

    async fn message(&self, ctx: Context, message: Message) {
        
        if message.content.starts_with("/") {
            return;
        }
        
        let gid = message.guild_id.unwrap();

        let data = ctx.data.read().await;
        let data = data.get::<GuildQueueContainer>().unwrap();
        let queue_lock = data.get(&gid).unwrap();

        let queue = queue_lock.read().await;
        if let Some(guild_chat_channel) = queue.chat_channel {
            if message.channel_id == guild_chat_channel {
                if let Some(track_handle) = queue.now_playing.clone() {
                    let content = message.content.to_lowercase();
                    if queue.skip_keyword.as_ref().unwrap().contains(&content) {
                        track_handle.stop().unwrap();
                    }
                }
            }
        }
    } 
}