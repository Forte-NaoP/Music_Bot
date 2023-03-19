use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption, CreateSelectMenuOptions},
    client::Context,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        id::GuildId,
        prelude::{
            Message,
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
    },
    framework::{
        standard::CommandResult,
    }
};

use std::any::Any;

pub enum CommandReturn {
    String(String),
    SingleEmbed(CreateEmbed),
    ControlInteraction(Box<dyn ControlInteraction + Send + Sync>),
    None,
}

#[async_trait]
pub trait ControlInteraction {
    async fn control_interaction(
        &mut self,
        ctx: &Context, 
        interaction: ApplicationCommandInteraction, 
    ) -> Result<(), serenity::Error>;
    fn as_any(&self) -> &dyn Any;
}