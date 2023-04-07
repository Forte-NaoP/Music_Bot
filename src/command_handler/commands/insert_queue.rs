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
    input::{self, Input},
    tracks::TrackHandle,
    Call,
};

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    }, 
    utils::{audio_module::youtube_dl::ytdl, guild_queue},
    GuildQueueContainer,
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

        let gid = command.guild_id.unwrap();
        let guild = ctx.cache.guild(gid).unwrap();

        {        
            let data = ctx.data.read().await;
            let data = data.get::<GuildQueueContainer>().unwrap();
            let queue = data.get(&gid).unwrap();
            match queue.try_write() {
                Ok(mut queue) => {
                    queue.url_queue.push_back(url.clone());
                },
                Err(_) => return CommandReturn::String("큐가 사용 중 입니다.".to_owned()),
            };
        }

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
