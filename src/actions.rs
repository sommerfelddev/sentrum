use std::fmt;

use anyhow::Result;
use async_scoped::TokioScope;
use async_trait::async_trait;
use log::{debug, info, warn};
use serde::Deserialize;

use crate::message::MessageConfig;
use crate::message::MessageParams;

mod command;
#[cfg(feature = "desktop")]
mod desktop_notification;
#[cfg(feature = "email")]
mod email;
#[cfg(feature = "nostr")]
mod nostr;
#[cfg(feature = "ntfy")]
mod ntfy;
#[cfg(feature = "telegram")]
mod telegram;
mod terminal_print;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum AnyActionConfig {
    TerminalPrint,
    Command(self::command::CommandConfig),
    #[cfg(feature = "desktop")]
    DesktopNotification,
    #[cfg(feature = "ntfy")]
    Ntfy(self::ntfy::NtfyConfig),
    #[cfg(feature = "email")]
    Email(self::email::EmailConfig),
    #[cfg(feature = "telegram")]
    Telegram(self::telegram::TelegramConfig),
    #[cfg(feature = "nostr")]
    Nostr(self::nostr::NostrConfig),
}

impl fmt::Display for AnyActionConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnyActionConfig::TerminalPrint => write!(f, "terminal_print"),
            AnyActionConfig::Command(_) => write!(f, "command"),
            #[cfg(feature = "desktop")]
            AnyActionConfig::DesktopNotification => write!(f, "desktop_notification"),
            #[cfg(feature = "ntfy")]
            AnyActionConfig::Ntfy(_) => write!(f, "ntfy"),
            #[cfg(feature = "email")]
            AnyActionConfig::Email(_) => write!(f, "email"),
            #[cfg(feature = "telegram")]
            AnyActionConfig::Telegram(_) => write!(f, "telegram"),
            #[cfg(feature = "nostr")]
            AnyActionConfig::Nostr(_) => write!(f, "nostr"),
        }
    }
}

#[async_trait]
pub trait Action<'a> {
    fn name(&self) -> &'static str;

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()>;
}

pub async fn get_action<'a>(
    message_config: &'a MessageConfig,
    action_config: &'a AnyActionConfig,
) -> Result<Box<dyn Action<'a> + 'a + Sync>> {
    Ok(match action_config {
        AnyActionConfig::TerminalPrint => Box::new(self::terminal_print::TerminalPrintAction::new(
            message_config,
        )),
        AnyActionConfig::Command(config) => {
            Box::new(self::command::CommandAction::new(message_config, config)?)
        }
        #[cfg(feature = "desktop")]
        AnyActionConfig::DesktopNotification => Box::new(
            self::desktop_notification::DesktopNotificationAction::new(message_config),
        ),
        #[cfg(feature = "ntfy")]
        AnyActionConfig::Ntfy(config) => {
            Box::new(self::ntfy::NtfyAction::new(message_config, config)?)
        }
        #[cfg(feature = "email")]
        AnyActionConfig::Email(config) => {
            Box::new(self::email::EmailAction::new(message_config, config)?)
        }
        #[cfg(feature = "telegram")]
        AnyActionConfig::Telegram(config) => {
            Box::new(self::telegram::TelegramAction::new(message_config, config)?)
        }
        #[cfg(feature = "nostr")]
        AnyActionConfig::Nostr(config) => {
            Box::new(self::nostr::NostrAction::new(message_config, config).await?)
        }
    })
}

pub async fn get_actions<'a>(
    message_config: &'a MessageConfig,
    actions_config: &'a [AnyActionConfig],
) -> Vec<Box<dyn Action<'a> + 'a + Sync>> {
    let mut result: Vec<Box<dyn Action + Sync>> = Default::default();

    // TODO: parallelize this. It's hard because the result vector needs to be shared.
    for action_config in actions_config {
        debug!("registering action '{}'", action_config);
        match get_action(message_config, action_config).await {
            Ok(action) => {
                info!("registered action '{}'", action_config);
                result.push(action);
            }
            Err(e) => {
                warn!("[{}] could not register action: {:#}", action_config, e);
            }
        }
    }

    result
}

pub async fn run_actions(
    actions: &[&(dyn Action<'_> + Sync)],
    params: Option<MessageParams<'_, '_>>,
) {
    TokioScope::scope_and_block(|s| {
        for &action in actions {
            debug!("running '{}' action", action.name());
            s.spawn(action.run(params.as_ref()));
        }
    });
}
