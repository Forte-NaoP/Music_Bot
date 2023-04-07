use songbird::{
    Call, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};

use serenity::{async_trait, client::Context, http::Http, model::id::ChannelId, prelude::RwLock};
use crate::utils::{
    guild_queue::GuildQueue,
    audio_module::youtube_dl::ytdl_optioned,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TrackEndNotifier {
    pub http: Arc<Http>,
    pub guild_queue: Arc<RwLock<GuildQueue>>,
    pub voice_manager: Arc<Mutex<Call>>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            println!("Song has ended: {:?}", track_list);
        }
        None
    }
}

pub struct TrackQueuingNotifier {
    pub http: Arc<Http>,
    pub guild_queue: Arc<RwLock<GuildQueue>>,
    pub voice_manager: Arc<Mutex<Call>>,
}

#[async_trait]
impl VoiceEventHandler for TrackQueuingNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            println!("Song has ended: {:?}", track_list);
        }
        {
            let (mut guild_queue, mut voice_manager) = 
                tokio::join!(self.guild_queue.write(), self.voice_manager.lock());
            if !guild_queue.url_queue.is_empty() {
                let url = guild_queue.url_queue.pop_front().unwrap();
                let input = ytdl_optioned(url, 10, 20).await.unwrap();
                let handle = voice_manager.play_source(input);
                guild_queue.now_playing = Some(handle);
            } else {
                voice_manager.remove_all_global_events();
                guild_queue.now_playing = None;
            }
        }
        None
    }
}