use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use ntfy::Auth;
use ntfy::Dispatcher;
use ntfy::Payload;
use ntfy::Priority;
use ntfy::Url;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use serde_json::from_reader;
use serde_json::to_string;

use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageFormat;
use crate::message::MessageParams;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NtfyData {
    topic: String,
}

fn get_random_topic() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

impl Default for NtfyData {
    fn default() -> Self {
        NtfyData {
            topic: get_random_topic(),
        }
    }
}

fn get_ntfy_data_filepath() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or(PathBuf::from("cache"))
        .join(env!("CARGO_PKG_NAME"))
        .join("ntfy.json")
}

fn get_ntfy_topic() -> Result<String> {
    let path = get_ntfy_data_filepath();
    Ok(match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            from_reader(reader)
                .with_context(|| format!("cannot read ntfy data from '{}'", path.display()))
        }
        Err(_) => {
            let ntfy_data = NtfyData::default();
            let mut file = File::create(&path)?;
            file.write_all(to_string(&ntfy_data)?.as_bytes())
                .with_context(|| format!("could not write ntfy data to '{}'", path.display()))?;
            Ok(ntfy_data)
        }
    }?
    .topic)
}

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

    pub fn topic(&self) -> Option<&str> {
        self.topic.as_deref()
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

        let topic = ntfy_config
            .topic()
            .unwrap_or(&get_ntfy_topic()?)
            .to_string();
        info!(
            "[ntfy] using topic '{}', connect to {}/{}",
            topic,
            ntfy_config.url(),
            topic
        );
        let mut payload = Payload::new(&topic)
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
