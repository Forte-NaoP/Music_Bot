use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework, CommandError, Args
    },
    model::{
        prelude::{Message, Reaction, ReactionType, Ready},
        application::{
            component::{SelectMenu, ComponentType, SelectMenuOption},
            interaction::{Interaction, InteractionResponseType},
            command::Command,
        },
        id::GuildId,
    },
};

use crate::{
    command_handler::{
        command_handler::*,
        commands::*,
        command_return::CommandReturn, self,
    },
};

use std::env;
use log::{error, info, warn};

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());

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
}