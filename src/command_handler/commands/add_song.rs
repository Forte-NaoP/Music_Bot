use std::{vec, collections::VecDeque};

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
    },
    utils::url_checker::{url_checker, UrlType},
    database_handler::*, DBContainer,
};

struct AddSong;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(AddSong)
}

#[async_trait]
impl CommandInterface for AddSong {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let gid = command.guild_id.unwrap();
        let url = match Option::<String>::from(DataWrapper::from(options, 0)) {
            Some(url) => url,
            None => return CommandReturn::String("url을 입력해주세요.".to_string()),
        };

        let title = match Option::<String>::from(DataWrapper::from(options, 1)) {
            Some(title) => title,
            None => return CommandReturn::String("제목을 입력해주세요.".to_string()),
        };

        let url = match url_checker(&url, UrlType::ID) {
            Some(url) => url,
            None => return CommandReturn::String("유효한 url이 아닙니다.".to_string()),
        };

        match add_title_with_url(&ctx.data.read().await.get::<DBContainer>().unwrap(), url.to_owned(), title.to_owned()).await {
            Ok(success) => {
                match success {
                    DBSuccess::NewUrl => {
                        CommandReturn::String(format!("새 곡 {}이 DB에 추가되었습니다.", title))
                    },
                    DBSuccess::ExistUrl => {
                        CommandReturn::String(format!("이미 DB에 존재하는 곡이므로 중복 제목으로 추가되었습니다."))
                    },
                }
            },
            Err(why) => {
                match why {
                    DBError::TitleAlreadyUsed => {
                        CommandReturn::String(format!("이미 사용 중인 제목입니다."))
                    },
                    _ => CommandReturn::String(format!("오류로 인해 곡이 등록되지 않았습니다."))
                }
            },
        }
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("곡추가")
            .description("DB에 곡을 추가합니다.")
            .create_option(|option| {
                option
                    .name("url")
                    .description("추가하고 싶은 노래 url")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("title")
                    .description("추가하고 싶은 노래 제목")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }
}