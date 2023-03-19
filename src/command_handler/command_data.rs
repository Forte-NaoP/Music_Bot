use serenity::{
    model::{
        prelude::{
            interaction::application_command::{CommandDataOption, CommandDataOptionValue},
        },
        user::User,
        channel::{PartialChannel, Attachment},
        guild::Role,
    },
};

pub struct DataWrapper {
    pub data: Option<CommandDataOptionValue>
}

impl DataWrapper {
    pub fn from(options: &[CommandDataOption], idx: usize) -> DataWrapper {
        let option = match options.get(idx) {
            Some(option) => match option.resolved.as_ref() {
                Some(option) => Some(option.to_owned()),
                None => None,
            },
            None => None,
        };
        DataWrapper { data: option }
    }
}

impl From<DataWrapper> for Option<String> {
    fn from(item: DataWrapper) -> Option<String> {
        match item.data {
            Some(CommandDataOptionValue::String(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<i64> {
    fn from(item: DataWrapper) -> Option<i64> {
        match item.data {
            Some(CommandDataOptionValue::Integer(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<bool> {
    fn from(item: DataWrapper) -> Option<bool> {
        match item.data {
            Some(CommandDataOptionValue::Boolean(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<User> {
    fn from(item: DataWrapper) -> Option<User> {
        match item.data {
            Some(CommandDataOptionValue::User(data, _)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<PartialChannel> {
    fn from(item: DataWrapper) -> Option<PartialChannel> {
        match item.data {
            Some(CommandDataOptionValue::Channel(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<Role> {
    fn from(item: DataWrapper) -> Option<Role> {
        match item.data {
            Some(CommandDataOptionValue::Role(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<f64> {
    fn from(item: DataWrapper) -> Option<f64> {
        match item.data {
            Some(CommandDataOptionValue::Number(data)) => Some(data),
            _ => None,
        }
    }
}

impl From<DataWrapper> for Option<Attachment> {
    fn from(item: DataWrapper) -> Option<Attachment> {
        match item.data {
            Some(CommandDataOptionValue::Attachment(data)) => Some(data),
            _ => None,
        }
    }
}