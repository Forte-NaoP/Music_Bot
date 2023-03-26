use std::{vec, collections::VecDeque};

use regex::Regex;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    client::{Context},
    framework::standard::{
        CommandResult, Args,
    },
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        id::{ChannelId, GuildId, UserId},
        prelude::{
            Message,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
            command::CommandOptionType,
        },
        user::User,
        guild::Guild,
    },
};
use songbird::{
    Songbird,
    input,
    tracks::create_player,
};

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    },
    MusicQueue,
};

struct InsertQueue;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(InsertQueue)
}

#[async_trait]
impl CommandInterface for InsertQueue {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let url = match Option::<String>::from(DataWrapper::from(options, 0)) {
            Some(url) => url,
            None => return CommandReturn::String("url을 입력해주세요.".to_string()),
        };

        let queue = ctx.data.read().await.get::<MusicQueue>().unwrap().clone();
        queue.write().await.push_back(url.to_owned());
        CommandReturn::String(format!("{}를 큐에 추가했습니다.", url))
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("큐잉")
            .description("큐에 곡을 추가합니다.")
            .create_option(|option| {
                option
                    .name("url")
                    .description("큐에 넣고 싶은 노래 url")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }
}
