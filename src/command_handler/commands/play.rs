use std::{vec, collections::VecDeque, time::Duration};

use tokio::time::sleep;
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

        let mut play_time = 0;
        let mut start_time = 0;
        
        let url = match Option::<String>::from(DataWrapper::from(options, 0)) {
            Some(url) => {
                play_time = match Option::<i64>::from(DataWrapper::from(options, 1)) {
                    Some(play_time) => play_time,
                    None => 0,
                };
        
                start_time = match Option::<i64>::from(DataWrapper::from(options, 2)) {
                    Some(start_time) => start_time,
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

        let mut handler_lock = voice_manager.get(gid).unwrap();
        let mut handler = handler_lock.lock().await;
        
        if let Some(url) = url {
            let src = match input::Restartable::ytdl(url, true).await {
                Ok(src) => src,
                Err(why) => {
                    println!("Err starting source: {:?}", why);
                    return CommandReturn::String("재생 못함".to_owned())
                }
            };
            let (mut audio, audio_handle) = create_player(src.into());
            audio_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier).unwrap();
            handler.play(audio);
            sleep(Duration::from_secs(30)).await;
            handler.stop();
        } else {
            while let Some(song_url) = queue.write().await.pop_front() {
                let src = match input::Restartable::ytdl(song_url, true).await {
                    Ok(src) => src,
                    Err(why) => {
                        println!("Err starting source: {:?}", why);
                        return CommandReturn::String("재생 못함".to_owned())
                    }
                };
                let (mut audio, audio_handle) = create_player(src.into());
                audio_handle.add_event(Event::Track(TrackEvent::End), TrackEndNotifier).unwrap();
                handler.play(audio);
                sleep(Duration::from_secs(30)).await;
                handler.stop();
            }
        }

        CommandReturn::String("와 정말 알고 싶던 정보였어요".to_owned())
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
