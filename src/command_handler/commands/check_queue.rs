use std::{vec, collections::VecDeque};

use regex::Regex;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
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

struct CheckQueue;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(CheckQueue)
}

#[async_trait]
impl CommandInterface for CheckQueue {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let guild = ctx.cache.guild(gid).unwrap();

        {        
            let data = ctx.data.read().await;
            let data = data.get::<GuildQueueContainer>().unwrap();
            let queue = data.get(&gid).unwrap();
            match queue.try_read() {
                Ok(queue) => {
                    let mut urls = vec![];
                    for url in queue.url_queue.iter() {
                        urls.push(url.as_str());
                    }
                    let url_list = urls.join("\n");
                    let mut embed = CreateEmbed::default();
                    embed.title("현재 재생중인 곡")
                        .description(format!("{}", url_list));
                    command.channel_id.send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.clone_from(&embed);
                            e
                        })
                    }).await.unwrap();
                },
                Err(_) => return CommandReturn::String("큐가 사용 중 입니다.".to_owned()),
            };
        }

        CommandReturn::None
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("큐확인")
            .description("큐에 있는 곡을 확인합니다.")
    }
}
