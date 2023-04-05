use std::{vec, collections::VecDeque, time::Duration, sync::Arc};

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


pub fn create_play_info_embed(title: &str, current: u64, total: u64) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("현재 재생중인 곡")
        .description(format!("{}\n{}/{}", title, duration_format(0), duration_format(total)));
    embed
}

pub fn update_play_info_embed(mut embed: CreateEmbed, title: &str, current: u64, total: u64) -> CreateEmbed {
    embed.description(format!("{}\n{}/{}", title, duration_format(current), duration_format(total)));
    embed
}

fn duration_format(duration: u64) -> String {
    let seconds = duration;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let seconds = seconds % 60;
    let minutes = minutes % 60;
    if hours == 0 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}