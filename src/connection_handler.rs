use serenity::{
    client::{Context},
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
    },
};
use songbird::{
    Songbird,
    error::JoinError,
};

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    },
    GuildQueueContainer,
};

#[derive(Debug)]
pub enum ConnectionErrorCode {
    AlreadyInUse,
    JoinVoiceChannelFirst,
    VoiceChannelNotFound,
    ServerNotFound,
    JoinError(JoinError)
}

pub enum ConnectionSuccessCode {
    AlreadyConnected,
    NewConnection,
}

pub async fn establish_connection(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<ConnectionSuccessCode, ConnectionErrorCode> {
    
    let gid = command.guild_id.unwrap();
    let guild = ctx.cache.guild(gid).unwrap();
    
    match guild.voice_states.get(&command.user.id).and_then(|vs| vs.channel_id) {
        // 사용자가 음성채널에 있을 때
        Some(user_ch_id) => {
            let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");
            match voice_manager.get(guild.id) {
                // 봇이 음성채널에 있을 때
                Some(call) => {
                    let locked_call = call.lock().await;
                    match locked_call.current_channel() {
                        Some(bot_ch_id) => {
                            if bot_ch_id.0 == user_ch_id.0 {
                                Ok(ConnectionSuccessCode::AlreadyConnected)
                            } else {
                                Err(ConnectionErrorCode::AlreadyInUse)
                            }
                        },
                        None => {
                            Err(ConnectionErrorCode::VoiceChannelNotFound)
                        }
                    }
                },
                None => {
                    let (_, join_result) = voice_manager.join(guild.id, user_ch_id).await;
                    match join_result {
                        Ok(()) => {
                            let gid = command.guild_id.unwrap();
                            let data = ctx.data.read().await;
                            let data = data.get::<GuildQueueContainer>().unwrap();
                            let queue_lock = data.get(&gid).unwrap();
                            {
                                let mut queue = queue_lock.write().await;
                                queue.chat_channel = Some(command.channel_id);
                            }
                            Ok(ConnectionSuccessCode::NewConnection)
                        },
                        Err(why) => {
                            Err(ConnectionErrorCode::JoinError(why))
                        }
                    }
                }
            }
        },
        None => Err(ConnectionErrorCode::JoinVoiceChannelFirst)
    }
}

pub async fn terminate_connection(ctx: &Context, command: &ApplicationCommandInteraction) {
    
    let gid = command.guild_id.unwrap();
    let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

    let handler_lock = voice_manager.get(gid).expect("Guild Not Found");
    handler_lock
        .lock()
        .await
        .leave()
        .await
        .expect("Disconnect Fail");

    let data = ctx.data.read().await;
    let data = data.get::<GuildQueueContainer>().unwrap();
    let queue_lock = data.get(&gid).unwrap();
    {
        let mut queue = queue_lock.write().await;
        queue.chat_channel = None;
    }

}