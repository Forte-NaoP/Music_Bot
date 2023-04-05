use std::{vec, collections::VecDeque, time::Duration, sync::Arc, fs::metadata};

use tokio::time::sleep;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    client::{
        Context,
    },
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
        Timestamp,
    },
    http::Http,
};
use songbird::{
    Songbird,
    input::{ffmpeg, ffmpeg_optioned, *},

    tracks::{create_player, PlayMode},
    EventHandler as VoiceEventHandler,
    EventContext,
    Event, TrackEvent,
};

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    },
    utils::{
        url_checker::{url_checker},
        audio_module::youtube_dl::{self, ytdl_optioned},
        play_info_notifier::{create_play_info_embed, update_play_info_embed}
    },
    connection_handler::*,
};

struct TrackEndNotifier;

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            println!("Song has ended: {:?}", track_list);
        }
        Some(Event::Track(TrackEvent::End))
    }
}

struct Play;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Play)
}

#[async_trait]
impl CommandInterface for Play {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let guild = ctx.cache.guild(gid).unwrap();
        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

        match establish_connection(&command.user.id, &guild, &voice_manager).await {
            Ok(success) => match success {
                ConnectionSuccessCode::AlreadyConnected => return CommandReturn::String("이미 음성채널에 접속되어 있습니다.".to_owned()),
                ConnectionSuccessCode::NewConnection => CommandReturn::String("음성채널에 접속했습니다.".to_owned())
            },
            Err(why) => match why {
                ConnectionErrorCode::JoinVoiceChannelFirst => return CommandReturn::String("음성채널에 먼저 접속해주세요.".to_owned()),
                ConnectionErrorCode::AlreadyInUse => return CommandReturn::String("다른 채널에서 사용중입니다.".to_owned()),
                _ => return CommandReturn::String("연결에 실패했습니다.".to_owned()),
            },
        };

        let mut play_time: u64 = 0;
        let mut start_time: u64 = 0;
        
        let url = match Option::<String>::from(DataWrapper::from(options, 0)) {
            Some(url) => {
                play_time = match Option::<i64>::from(DataWrapper::from(options, 1)) {
                    Some(play_time) => play_time.abs() as u64,
                    None => 0,
                };
        
                start_time = match Option::<i64>::from(DataWrapper::from(options, 2)) {
                    Some(start_time) => start_time.abs() as u64,
                    None => 0,
                };
                Some(url)
            },
            None => return CommandReturn::String("큐가 비어있습니다.\n큐에 곡을 넣거나 url을 입력해주세요.".to_string())
        };

        let http = ctx.http.clone();
        let current_channel = command.channel_id.clone();
        let handler_lock = voice_manager.get(gid).unwrap();

        if let Some(url) = url {
            let url = match url_checker(url.as_str()) {
                Some(url) => url,
                None => return CommandReturn::String("url이 잘못되었습니다.".to_string()),
            };
            let src = ytdl_optioned(url, start_time, play_time).await.unwrap();
            let metadata = src.metadata.clone();
            let title = metadata.title.unwrap();
            let duration = if play_time != 0 {
                play_time
            } else {
                metadata.duration.unwrap().as_secs()
            };

            let mut handler = handler_lock.lock().await;
            let mut audio_handle = handler.enqueue_source(src);
            audio_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier).unwrap();
            let mut last_edit_time = audio_handle.get_info().await.unwrap().position;

            let mut embed = create_play_info_embed(title.as_ref(), 0, duration);
            let mut msg = current_channel.send_message(&http, |m| {
                m.embed(|e| {
                    e.clone_from(&embed);
                    e
                })
            }).await.unwrap();

            // while let Ok(playing_info) = audio_handle.get_info().await {
            //     let current_time = playing_info.position;
            //     if current_time - last_edit_time >= Duration::from_secs(1) {
            //         embed = update_play_info_embed(embed, title.as_ref(), current_time.as_secs(), duration);
            //         msg.edit(&http, |m| {
            //             m.embed(|e| {
            //                 e.clone_from(&embed);
            //                 e
            //             })
            //         }).await.unwrap();
            //         last_edit_time = current_time;
            //     }
            // }

        } else {

        }
        CommandReturn::String("재생종료".to_owned())
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("재생")
            .description("재생")
            .create_option(|option| {
                option
                    .name("url")
                    .description("재생하고 싶은 노래 url")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("재생시간")
                    .description("재생시간")
                    .kind(CommandOptionType::Integer)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("시작시간")
                    .description("시작시간(초)")
                    .kind(CommandOptionType::Integer)
                    .required(false)
            })
    }
}