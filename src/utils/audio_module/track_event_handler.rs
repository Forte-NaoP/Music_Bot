use songbird::{
    Call, Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent,
};

use serenity::{async_trait, client::Context, http::Http, model::id::ChannelId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TrackEndNotifier;

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            println!("Song has ended: {:?}", track_list);
        }
        None
    }
}