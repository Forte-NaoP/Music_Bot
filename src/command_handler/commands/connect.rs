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
use songbird::Songbird;

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    }
};

struct Help;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Help)
}

#[async_trait]
impl CommandInterface for Help {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();

        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

        match connection_check(&command.user.id, &ctx.cache.guild(&gid), &voice_manager).await {
            Some(r) => {
                let (voice_node, voice_check) = voice_manager.join(gid.clone(), r).await;
            },
            None => ()
        }

        CommandReturn::String("와 정말 알고 싶던 정보였어요".to_owned())
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("접속")
            .description("접속")
    }
}

async fn connection_check(uid: &UserId, guild: &Option<Guild>, voice_manager: &Songbird) -> Option<ChannelId>{
    return match guild {
        Some(g) => {
            match g.voice_states.get(uid).and_then(|vs| vs.channel_id) {
                Some(user_ch_id) => { //유저가 음성채널에 있을떄
                    match voice_manager.get(g.id) {
                        Some(call) => {
                            None
                        },
                        None => Some(user_ch_id) // 음성채널에 연결 가능
                    }
                },
                None => None // 유저가 음성채널에 없을때
            }
        },
        None => None // 길드 정보 없을때
    }
}