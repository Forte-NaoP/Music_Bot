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
        audio_module::{
            youtube_dl::ytdl_optioned,
            track_event_handler::TrackEndNotifier,
        },
        play_info_notifier::{create_play_info_embed, update_play_info_embed}
    },
    connection_handler::*,
};

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
        
        let url = Option::<String>::from(DataWrapper::from(options, 0)).unwrap();

        let play_time = match Option::<i64>::from(DataWrapper::from(options, 1)) {
            Some(play_time) => play_time.abs() as u64,
            None => 0,
        };

        let start_time = match Option::<i64>::from(DataWrapper::from(options, 2)) {
            Some(start_time) => start_time.abs() as u64,
            None => 0,
        };

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

        let handler_lock = voice_manager.get(gid).unwrap();
        let mut handler = handler_lock.lock().await;

        let audio_handle = handler.enqueue_source(src);
        audio_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier).unwrap();

        let mut current_time = Duration::from_secs(0);
        let mut last_edit_time = audio_handle.get_info().await.unwrap().position;

        let mut embed = create_play_info_embed(title.as_ref(), 0, duration);
        let mut msg = command.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.clone_from(&embed);
                e
            })
        }).await.unwrap();

        while let Ok(playing_info) = audio_handle.get_info().await {
            current_time = playing_info.position;
            if current_time - last_edit_time >= Duration::from_millis(900) {
                embed = update_play_info_embed(embed, title.as_ref(), current_time.as_secs()+1, duration);
                msg.edit(&ctx.http, |m| {
                    m.embed(|e| {
                        e.clone_from(&embed);
                        e
                    })
                }).await.unwrap();
                last_edit_time = current_time;
            }
            if current_time >= Duration::from_secs(duration) {
                embed = update_play_info_embed(embed, title.as_ref(), duration, duration);
                msg.edit(&ctx.http, |m| {
                    m.embed(|e| {
                        e.clone_from(&embed);
                        e
                    })
                }).await.unwrap();
                break;
            }
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
                    .required(true)
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