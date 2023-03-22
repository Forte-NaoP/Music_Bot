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
    }, connection_handler::terminate_connection
};

struct Disconnect;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(Disconnect)
}

#[async_trait]
impl CommandInterface for Disconnect {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let guild = ctx.cache.guild(gid).unwrap();

        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

        terminate_connection(&guild, &voice_manager).await;
        CommandReturn::String("접속을 종료합니다.".to_owned())
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("퇴장")
            .description("퇴장")
    }
}