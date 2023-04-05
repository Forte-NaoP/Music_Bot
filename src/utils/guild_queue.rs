use serenity::client::Context;
use serenity::model::{
    guild::Guild,
    id::{ChannelId, GuildId},
};

use songbird::input::Input;
use songbird::tracks::TrackHandle;
use songbird::input::Metadata;

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::Mutex;

use crate::GuildQueueContainer;
pub struct GuildQueue {
    pub gid: GuildId,
    pub voice_channel: Option<ChannelId>,
    pub chat_channel: Option<ChannelId>,
    //pub src_queue: Arc<RwLock<VecDeque<Input>>>,
    pub url_queue: Box<VecDeque<String>>,
    pub now_playing: Option<TrackHandle>,
}

impl GuildQueue {
    fn new(gid: GuildId) -> GuildQueue {
        GuildQueue {
            gid,
            voice_channel: None,
            chat_channel: None,
            //src_queue: Arc::new(RwLock::new(VecDeque::new())),
            url_queue: Box::new(VecDeque::new()),
            now_playing: None,
        }
    }

    fn skip(&mut self) {
        if let Some(track_handle) = self.now_playing.as_mut() {
            track_handle.stop().unwrap();
        }
    }
}

pub async fn initialize(ctx: &Context) {
    let _guild_queue = ctx.data.read().await;
    let guild_queue = _guild_queue.get::<GuildQueueContainer>().unwrap();
    let mut guild_queue_lock = guild_queue.write().await;
    for gid in ctx.cache.guilds().iter() {
        guild_queue_lock.entry(gid.clone()).or_insert(GuildQueue::new(gid.clone()));
    }
}