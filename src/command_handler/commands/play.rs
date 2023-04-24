use std::{
    time::Duration,
    sync::Arc,
};

use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    client::{
        Context,
    },
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        id::{ChannelId, GuildId, UserId},
        prelude::{
            Message,
            interaction::application_command::{CommandDataOption},
            command::CommandOptionType,
        },
    },
    http::Http,
};
use songbird::{
    Event, TrackEvent,
};
use std::time::{Instant};
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
        play_info_notifier::{create_play_info_embed, update_play_info_embed}
    },
    connection_handler::*,
    GuildQueueContainer
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

        match establish_connection(ctx, command).await {
            Ok(_) => (),
            Err(why) => match why {
                ConnectionErrorCode::JoinVoiceChannelFirst => return CommandReturn::String("음성채널에 먼저 접속해주세요.".to_owned()),
                ConnectionErrorCode::AlreadyInUse => return CommandReturn::String("다른 채널에서 사용중입니다.".to_owned()),
                _ => return CommandReturn::String("연결에 실패했습니다.".to_owned()),
            },
        };

        let gid = command.guild_id.unwrap();
        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");    
        
        let url = Option::<String>::from(DataWrapper::from(options, 0)).unwrap();

        let play_time = match Option::<i64>::from(DataWrapper::from(options, 1)) {
            Some(play_time) => play_time.abs() as u64,
            None => 0,
        };

        let start_time = match Option::<i64>::from(DataWrapper::from(options, 2)) {
            Some(start_time) => start_time.abs() as u64,
            None => 0,
        };

        let skip_keyword = Option::<String>::from(DataWrapper::from(options, 3));

        let url = match url_checker(url.as_str()) {
            Some(url) => url,
            None => return CommandReturn::String("url이 잘못되었습니다.".to_string()),
        };

        let start = Instant::now();
        let src = ytdl_optioned(url, start_time, play_time).await.unwrap();
        let d = start.elapsed();
        println!("{:?} elapsed", d);

        let metadata = src.metadata.clone();
        let title = metadata.title.unwrap();
        let duration = if play_time != 0 {
            play_time
        } else {
            metadata.duration.unwrap().as_secs()
        };

        let handler_lock = voice_manager.get(gid).unwrap();
        let mut handler = handler_lock.lock().await;

        let data = ctx.data.read().await;
        let data = data.get::<GuildQueueContainer>().unwrap();
        let queue_lock = data.get(&gid).unwrap();
        let audio_handle = Arc::new(handler.enqueue_source(src));
        {
            let mut queue = queue_lock.write().await;
            handler.add_global_event(Event::Track(TrackEvent::End), 
                TrackEndNotifier {
                    http: ctx.http.clone(),
                    guild_queue: queue_lock.clone(),
                    voice_manager: handler_lock.clone(),
                }
            );
            queue.now_playing = Some(audio_handle.clone());
            if let Some(skip_keyword) = skip_keyword {
                let skip_keyword = skip_keyword.split(',')
                    .map(|s| s.to_string().trim().to_string())
                    .collect::<Vec<String>>();
                println!("skip_keyword {:?}", skip_keyword);
                queue.skip_keyword = Some(skip_keyword);
            }
        }  

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
            .create_option(|option| {
                option
                    .name("스킵")
                    .description("곡을 스킵할 때 사용할 키워드 ,(콤마)로 구분")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }
}