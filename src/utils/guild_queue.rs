use serenity::client::Context;
use serenity::model::{
    guild::Guild,
    id::{ChannelId, GuildId},
};

use songbird::tracks::TrackHandle;

use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::RwLock;

use crate::GuildQueueContainer;
pub struct GuildQueue {
    pub gid: GuildId,

    // the channel where the bot will join
    pub voice_channel: Option<ChannelId>,

    // the channel where the bot observes the chat
    pub chat_channel: Option<ChannelId>,
    
    // the queue of songs
    pub url_queue: Box<VecDeque<String>>,

    // the current song
    pub now_playing: Option<Arc<TrackHandle>>,

    // keyword for skipping the current song
    pub skip_keyword: Option<Vec<String>>,
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
            skip_keyword: None,
        }
    }

    fn skip(&mut self) {
        if let Some(track_handle) = self.now_playing.as_mut() {
            track_handle.stop().unwrap();
        }
    }
}

pub async fn initialize(ctx: &Context) {
    let mut guild_queue = ctx.data.write().await;
    let guild_queue = guild_queue.get_mut::<GuildQueueContainer>().unwrap();
    for gid in ctx.cache.guilds().iter() {
        guild_queue.entry(gid.clone()).or_insert(Arc::new(RwLock::new(GuildQueue::new(gid.clone()))));
    }
}