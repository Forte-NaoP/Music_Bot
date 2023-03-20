use regex::Regex;
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
use songbird::{
    Songbird,
    input,
    tracks::create_player,
};

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

        let url = Option::<String>::from(DataWrapper::from(options, 0)).unwrap();

        let url_regex = Regex::new(r"^https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&/=]*)$").unwrap();

        if !url_regex.is_match(url.as_str()) {
            return CommandReturn::String("주소 좀 똑바로 쳐라".to_owned())
        }

        let gid = command.guild_id.unwrap();
        let voice_manager = songbird::get(ctx).await.expect("Songbird Voice client placed in at initialisation.");

        if let Some(handler_lock) = voice_manager.get(gid) {
            let mut handler = handler_lock.lock().await;
            let src = match input::Restartable::ytdl(url, true).await {
                Ok(src) => src,
                Err(why) => {
                    println!("Err starting source: {:?}", why);
                    return CommandReturn::String("재생 못함".to_owned())
                }
            };
            let (mut audio, audio_handle) = create_player(src.into());

            println!("{}", handler.current_channel().unwrap());
            audio_handle.enable_loop().expect("restartable err");
            handler.play(audio);
            
        }

        CommandReturn::String("와 정말 알고 싶던 정보였어요".to_owned())
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("재생")
            .description("재생")
            .create_option(|option| {
                option
                    .name("url")
                    .description("재생하고 싶은 노래 url")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }
}
