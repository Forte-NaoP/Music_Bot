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
    GuildQueueContainer,
};

struct Stat;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Stat)
}

#[async_trait]
impl CommandInterface for Stat {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let data_lock = ctx.data.read().await;
        let data = data_lock.get::<GuildQueueContainer>().unwrap();
        let queue_lock = data.get(&gid).unwrap();
        let queue = queue_lock.read().await;

        let voice_name = match queue.voice_channel {
            Some(vc) => vc.name(&ctx.cache).await.unwrap(),
            None => "None".to_string(),
        };

        let text_name = match queue.chat_channel {
            Some(tc) => tc.name(&ctx.cache).await.unwrap(),
            None => "None".to_string(),
        };

        let mut embed = CreateEmbed::default();
        embed.title("접속 중인 채널");
        embed.description(format!("음성채널: {}\n채팅채널: {}", voice_name, text_name));


        CommandReturn::SingleEmbed(embed)
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("스탯")
            .description("스탯")
    }
}