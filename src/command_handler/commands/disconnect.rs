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

        voice_manager.remove(gid).await.expect("Songbird Voice client disconnect fail.");

        CommandReturn::String("난 여길 빠져나가야 겠어".to_owned())
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