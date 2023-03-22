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
    },
    connection_handler::*,
};

struct Connect;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Connect)
}

#[async_trait]
impl CommandInterface for Connect {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let guild = ctx.cache.guild(gid).unwrap();

        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

        match establish_connection(&command.user.id, &guild, &voice_manager).await {
            Ok(success) => match success {
                ConnectionSuccessCode::AlreadyConnected => CommandReturn::String("이미 음성채널에 접속되어 있습니다.".to_owned()),
                ConnectionSuccessCode::NewConnection => CommandReturn::String("음성채널에 접속했습니다.".to_owned())
            },
            Err(why) => match why {
                ConnectionErrorCode::JoinVoiceChannelFirst => CommandReturn::String("음성채널에 먼저 접속해주세요.".to_owned()),
                ConnectionErrorCode::AlreadyInUse => CommandReturn::String("다른 채널에서 사용중입니다.".to_owned()),
                _ => CommandReturn::String("연결에 실패했습니다.".to_owned()),
            },
        }
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