use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::Context,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        id::GuildId,
        prelude::{
            Message,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
    },
    framework::{
        standard::CommandResult,
    }
};

use crate::{
    command_handler::{
        commands, 
        command_return::{CommandReturn, ControlInteraction}
    }, 
};

use std::collections::HashMap;
use std::sync::Arc;
use lazy_static::lazy_static;
use log::error;

#[async_trait]
pub trait CommandInterface {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn;
    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand;
}

pub struct CommandList {
    pub commands: HashMap<&'static str, Box<dyn CommandInterface + Send + Sync>>,
}

impl CommandList {
    pub async fn register(&'static self, gid: GuildId, ctx: &Context) {
        for (_, command) in &self.commands {
            if let Err(why) = gid
                .create_application_command(&ctx.http, |c| command.register(c))
                .await
            {
                println!("Cannot create application command: {:#?}", why);
            }
        }
    }
}

lazy_static! {
    pub static ref COMMAND_LIST: CommandList = CommandList {
        commands: HashMap::from([
            ("도움말", commands::help::command()),
            ("접속", commands::connect::command()),
        ])
    };
}

pub async fn execute_command(ctx: &Context, command: ApplicationCommandInteraction) {

    command.defer(&ctx.http).await.unwrap();

    let cmd_result = match COMMAND_LIST.commands.get(command.data.name.as_str()) {
        Some(result) => result.run(&ctx, &command, &command.data.options).await,
        None => CommandReturn::String("등록되지않은 명령어입니다.".to_string()),
    };

    match cmd_result {
        CommandReturn::String(content) => {
            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |msg| msg.content(&content))
                .await
            {
                error!(
                    "Failed to send Single-string \"{:?}\" from command \"{}\".",
                    content, command.data.name
                );
                error!("{:#?}", why);
            }
        }
        CommandReturn::SingleEmbed(embed) => {
            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |msg| msg.set_embed(embed.clone()))
                .await
            {
                error!(
                    "Failed to send single-embed \"{:#?}\" from command \"{}\".",
                    embed, command.data.name
                );
                error!("{:#?}", why);
            }
        }
        CommandReturn::ControlInteraction(mut pages) => {          
            if let Err(why) = pages.control_interaction(ctx, command).await {
                error!("an error occured while handling embed pages.");
                error!("{:#?}", why);
            }
        }
        _ => ()
    }

}

