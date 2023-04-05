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
    Call,
};

use crate::{
    command_handler::{
        command_handler::*,
        command_data::*,
        command_return::CommandReturn,
    },
};

struct InsertQueue;

pub fn command() -> Box<dyn CommandInterface + Sync + Send> {
    Box::new(InsertQueue)
}

#[async_trait]
impl CommandInterface for InsertQueue {
    async fn run(
        &self, 
        ctx: &Context, 
        command: &ApplicationCommandInteraction, 
        options: &[CommandDataOption]
    ) -> CommandReturn {

        let url = match Option::<String>::from(DataWrapper::from(options, 0)) {
            Some(url) => url,
            None => return CommandReturn::String("url을 입력해주세요.".to_string()),
        };

        let start_imidately = match Option::<bool>::from(DataWrapper::from(options, 1)) {
            Some(start_imidately) => start_imidately,
            None => false,
        };

        CommandReturn::String(format!("{}를 큐에 추가했습니다.", url))
    }

    fn register<'a: 'b, 'b>(
        &'a self,
        command: &'a mut CreateApplicationCommand
    ) -> &'b mut CreateApplicationCommand {
        command
            .name("큐잉")
            .description("큐에 곡을 추가합니다.")
            .create_option(|option| {
                option
                    .name("url")
                    .description("큐에 넣고 싶은 노래 url")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("바로 재생")
                    .description("큐에 추가하고 바로 재생합니다. (Default: false)")
                    .kind(CommandOptionType::Boolean)
                    .required(false)
            })
    }
}

// guildqueue.url_queueu에서 src_queue로 ytdl로 input 넣기
// 단 src_queue는 최대 10개 까지
// 재생을 시작하면 스레드 하나 만들어서 무한루프 돌면서 src_queue 길이 10인지 체크하면서 url_queue에서 가져오기
// 재생하는 곳에서는 src_queue에서 enqueue_source로 재생큐에 넣기
// 모든 재생 끝나면 src_queue 스레드 종료 시키기


async fn convert_url_to_src(ctx: &Context, gid: GuildId, voice_manager: &Songbird) {
    use crate::utils::audio_module::youtube_dl;
    use crate::GuildQueueContainer;
    
    let _gqc = ctx.data.read().await;
    let gqc = _gqc.get::<GuildQueueContainer>().unwrap();

    let _gq = gqc.write().await;
    let gq = _gq.get_mut(&gid).unwrap();

    let handler = voice_manager.get(gid).unwrap();
    let handler_lock = handler.lock().await;

    while handler_lock.queue().len() < 10 {
        let url = gq.url_queue.pop_front().unwrap();
        let src = youtube_dl::ytdl(url).await.unwrap();
        handler_lock.enqueue_source(src);
    }


}