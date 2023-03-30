use std::{vec, collections::VecDeque, time::Duration, sync::Arc};

use tokio::time::sleep;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
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

    tracks::create_player,
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
    MusicQueue,
    utils::{
        url_checker::{url_checker},
        audio_module::youtube_dl::{self, ytdl_optioned},
    }
};

struct TrackEndNotifier;

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            println!("Song has ended: {:?}", track_list);
        }
        None
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
        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");
        let queue = ctx.data.read().await.get::<MusicQueue>().unwrap().clone();

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
            None => if queue.read().await.len() == 0 {
                return CommandReturn::String("큐가 비어있습니다.\n큐에 곡을 넣거나 url을 입력해주세요.".to_string())
            } else {
                None
            },
        };

        let http = ctx.http.clone();
        let current_channel = command.channel_id.clone();
        let mut handler_lock = voice_manager.get(gid).unwrap();

        if let Some(url) = url {
            let url = match url_checker(url.as_str()) {
                Some(url) => url,
                None => return CommandReturn::String("url이 잘못되었습니다.".to_string()),
            };

            //let src = ytdl(url).await.unwrap();
            let src = ytdl_optioned(url, start_time.to_string(), play_time.to_string()).await.unwrap();
            println!("play_time {}, start__time {}", play_time, start_time);
            let (mut audio, audio_handle) = create_player(src.into());
            let mut handler = handler_lock.lock().await;
            audio_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier).unwrap();
            println!("{:?}", audio_handle.metadata());
            println!("{:?}", audio);
            handler.play(audio);

        } else {
            
            
        }
        CommandReturn::String("재생 종료".to_owned())
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
