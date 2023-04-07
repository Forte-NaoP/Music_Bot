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

    tracks::{create_player, PlayMode, TrackHandle},
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
            track_event_handler::{TrackEndNotifier, TrackQueuingNotifier},
        },
        play_info_notifier::{create_play_info_embed, update_play_info_embed}, guild_queue::GuildQueue
    },
    connection_handler::*, GuildQueueContainer,
};

struct PlayQueue;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(PlayQueue)
}

#[async_trait]
impl CommandInterface for PlayQueue {
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
        
        let mut url: Option<String> = None;
        let mut now_plyaing: Option<TrackHandle> = None;
        {        
            let data = ctx.data.read().await;
            let data = data.get::<GuildQueueContainer>().unwrap();
            let queue = data.get(&gid).unwrap();
            match queue.try_write() {
                Ok(mut queue) => {
                    now_plyaing = queue.now_playing.clone();
                    url = queue.url_queue.pop_front();
                },
                Err(_) => return CommandReturn::String("큐가 사용 중 입니다.".to_owned()),
            };
        }

        match now_plyaing {
            Some(_) => {
                return CommandReturn::String("재생중인 곡이 있습니다.".to_owned());
            },
            None => match url {
                Some(url) => {
                    let src = ytdl_optioned(url, 10, 20).await.unwrap();
                    let handler_lock = voice_manager.get(gid).unwrap();
                    let mut handler = handler_lock.lock().await;
            
                    let data = ctx.data.read().await;
                    let data = data.get::<GuildQueueContainer>().unwrap();
                    let queue_lock = data.get(&gid).unwrap();
                    {
                        let mut queue = queue_lock.write().await;
                        let audio_handle = handler.enqueue_source(src);
                        handler.add_global_event(Event::Track(TrackEvent::End), 
                            TrackQueuingNotifier {
                                http: ctx.http.clone(),
                                guild_queue: queue_lock.clone(),
                                voice_manager: handler_lock.clone(),
                            }
                        );
                        queue.now_playing = Some(audio_handle);
                    }                    
                },
                None => return CommandReturn::String("재생할 곡이 없습니다.".to_owned()),
            },
        }
        CommandReturn::None
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("큐재생")
            .description("큐재생")
    }
}

