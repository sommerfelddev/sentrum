use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageFormat;
use crate::message::MessageParams;
use anyhow::Result;
use async_trait::async_trait;
use ntfy::Auth;
use ntfy::Dispatcher;
use ntfy::Payload;
use ntfy::Priority;
use ntfy::Url;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(remote = "Priority")]
#[serde(rename_all = "snake_case")]
pub enum NtfyPriority {
    Max = 5,
    High = 4,
    Default = 3,
    Low = 2,
    Min = 1,
}

#[derive(Deserialize, Debug)]
pub struct NtfyCredentials {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct NtfyConfig {
    url: Option<String>,
    proxy: Option<String>,
    topic: Option<String>,
    pub credentials: Option<NtfyCredentials>,
    #[serde(with = "NtfyPriority")]
    #[serde(default)]
    pub priority: Priority,
    pub tags: Option<Vec<String>>,
    pub attach: Option<Url>,
    pub filename: Option<String>,
    pub delay: Option<String>,
    pub email: Option<String>,
}

impl NtfyConfig {
    pub fn url(&self) -> &str {
        self.url.as_deref().unwrap_or("https://ntfy.sh")
    }

    pub fn topic(&self) -> &str {
        self.topic.as_deref().unwrap_or(env!("CARGO_PKG_NAME"))
    }
}

pub struct NtfyAction<'a> {
    message_config: &'a MessageConfig,
    dispatcher: Dispatcher,
    payload_template: Payload,
}

impl<'a> NtfyAction<'a> {
    pub fn new(message_config: &'a MessageConfig, ntfy_config: &'a NtfyConfig) -> Result<Self> {
        let mut dispatcher_builder = Dispatcher::builder(ntfy_config.url());
        if let Some(cred) = &ntfy_config.credentials {
            dispatcher_builder =
                dispatcher_builder.credentials(Auth::new(&cred.username, &cred.password));
        }
        if let Some(proxy) = &ntfy_config.proxy {
            dispatcher_builder = dispatcher_builder.proxy(proxy);
        }

        let mut payload = Payload::new(ntfy_config.topic())
            .markdown(match message_config.format() {
                MessageFormat::Plain => false,
                MessageFormat::Markdown => true,
                MessageFormat::Html => true,
            })
            .priority(ntfy_config.priority.clone())
            .tags(
                ntfy_config
                    .tags
                    .as_deref()
                    .unwrap_or(&["rotating_light".to_string()]),
            );
        if let Some(attach) = &ntfy_config.attach {
            payload = payload.attach(attach.clone());
        }
        if let Some(filename) = &ntfy_config.filename {
            payload = payload.filename(filename.clone());
        }
        if let Some(delay) = &ntfy_config.delay {
            payload = payload.delay(delay.parse()?);
        }
        if let Some(email) = &ntfy_config.email {
            payload = payload.email(email.clone());
        }
        Ok(Self {
            message_config,
            dispatcher: dispatcher_builder.build()?,
            payload_template: payload,
        })
    }
}

#[async_trait]
impl Action<'_> for NtfyAction<'_> {
    fn name(&self) -> &'static str {
        "ntfy"
    }

    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        let payload = self
            .payload_template
            .clone()
            .title(self.message_config.subject(params)?)
            .message(self.message_config.body(params)?)
            .click(self.message_config.get_tx_url(params)?.parse()?);
        self.dispatcher.send(&payload).await?;
        Ok(())
    }
}
