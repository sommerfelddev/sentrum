use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageParams;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use teloxide::requests::Requester;
use teloxide::types::UserId;
use teloxide::Bot;

#[derive(Deserialize, Debug)]
pub struct TelegramConfig {
    bot_token: String,
    user_id: u64,
}

impl TelegramConfig {
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}

pub struct TelegramAction<'a> {
    message_config: &'a MessageConfig,
    bot: Bot,
    user_id: UserId,
}

impl<'a> TelegramAction<'a> {
    pub fn new(
        message_config: &'a MessageConfig,
        telegram_config: &'a TelegramConfig,
    ) -> Result<Self> {
        Ok(Self {
            message_config,
            bot: Bot::new(telegram_config.bot_token()),
            user_id: UserId(telegram_config.user_id()),
        })
    }
}

#[async_trait]
impl Action<'_> for TelegramAction<'_> {
    fn name(&self) -> &'static str {
        "telegram"
    }

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        let subject = self.message_config.subject(params)?;
        let body = self.message_config.body(params)?;
        self.bot
            .send_message(self.user_id, format!("{}\n{}", subject, body))
            .await?;
        Ok(())
    }
}
