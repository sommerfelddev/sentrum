use super::Action;
use crate::message::MessageConfig;
use crate::message::MessageParams;
use anyhow::{Context, Result};
use async_scoped::TokioScope;
use async_trait::async_trait;
use const_format::formatcp;
use nostr_relay_pool::RelayOptions;
use nostr_sdk::nips::nip05;
use nostr_sdk::serde_json::from_reader;
use nostr_sdk::serde_json::to_string;
use nostr_sdk::Client;
use nostr_sdk::Keys;
use nostr_sdk::Metadata;
use nostr_sdk::PublicKey;
use nostr_sdk::ToBech32;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NostrData {
    key: String,
    metadata_set: bool,
}

impl Default for NostrData {
    fn default() -> Self {
        NostrData {
            key: Keys::generate().secret_key().unwrap().to_bech32().unwrap(),
            metadata_set: false,
        }
    }
}

fn get_nostr_data_filepath() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or(PathBuf::from("cache"))
        .join(env!("CARGO_PKG_NAME"))
        .join("nostr.json")
}

fn get_nostr_data() -> Result<NostrData> {
    let path = get_nostr_data_filepath();
    match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            from_reader(reader)
                .with_context(|| format!("cannot read nostr data from '{}'", path.display()))
        }
        Err(_) => {
            let nostr_data = NostrData::default();
            let mut file = File::create(&path)?;
            file.write_all(to_string(&nostr_data)?.as_bytes())
                .with_context(|| format!("could not write nostr data to '{}'", path.display()))?;
            Ok(nostr_data)
        }
    }
}

fn get_default_relays() -> Vec<String> {
    vec![
        "wss://nostr.bitcoiner.social",
        "wss://nostr.oxtr.dev",
        "wss://nostr.orangepill.dev",
        "wss://relay.damus.io",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn get_default_bot_metadata() -> Metadata {
    Metadata::new()
        .name(formatcp!("{}bot", env!("CARGO_PKG_NAME")))
        .display_name(formatcp!("{} bot", env!("CARGO_PKG_NAME")))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .website(env!("CARGO_PKG_REPOSITORY").parse().unwrap())
        .picture("https://robohash.org/sentrumbot.png".parse().unwrap())
        .banner(
            "https://void.cat/d/HX1pPeqz21hvneLDibs5JD.webp"
                .parse()
                .unwrap(),
        )
        .lud06(formatcp!(
            "https://sommerfeld.dev/.well-known/lnurlp/{}",
            env!("CARGO_PKG_NAME")
        ))
        .lud16(formatcp!("{}@sommerfeld.dev", env!("CARGO_PKG_NAME")))
}

fn mark_bot_metadata_as_set(mut nostr_data: NostrData) -> Result<()> {
    let path = get_nostr_data_filepath();
    nostr_data.metadata_set = true;
    let mut file = File::create(&path)?;
    file.write_all(to_string(&nostr_data)?.as_bytes())
        .with_context(|| format!("could not write nostr data to '{}'", path.display()))?;
    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct NostrConfig {
    #[serde(default = "get_default_relays")]
    relays: Vec<String>,
    proxy: Option<SocketAddr>,
    #[serde(default = "get_default_bot_metadata")]
    bot_metadata: Metadata,
    #[serde(default)]
    resend_bot_metadata: bool,
    recipient: String,
    #[serde(default)]
    sealed_dm: bool,
}

impl NostrConfig {}

pub struct NostrAction<'a> {
    message_config: &'a MessageConfig,
    client: Client,
    recipient: PublicKey,
    sealed_dm: bool,
}

impl<'a> NostrAction<'a> {
    pub async fn new(
        message_config: &'a MessageConfig,
        nostr_config: &'a NostrConfig,
    ) -> Result<Self> {
        let nostr_data = get_nostr_data()?;
        let keys = Keys::parse(&nostr_data.key)
            .with_context(|| format!("could not parse nostr secret key '{}'", nostr_data.key))?;

        let client = Client::new(&keys);

        let relay_opts = RelayOptions::new().read(false).proxy(nostr_config.proxy);
        TokioScope::scope_and_block(|s| {
            for relay in nostr_config.relays.iter() {
                s.spawn(client.add_relay_with_opts(relay.clone(), relay_opts.clone()));
            }
        });

        client.connect().await;

        if !nostr_data.metadata_set || nostr_config.resend_bot_metadata {
            client.set_metadata(&nostr_config.bot_metadata).await?;
            mark_bot_metadata_as_set(nostr_data)?;
        }

        let recipient = match PublicKey::parse(&nostr_config.recipient) {
            Ok(p) => p,
            Err(e) => {
                nip05::get_profile(&nostr_config.recipient, nostr_config.proxy)
                    .await
                    .with_context(|| {
                        format!("invalid recipient '{}': {}", nostr_config.recipient, e)
                    })?
                    .public_key
            }
        };

        Ok(Self {
            message_config,
            client,
            recipient,
            sealed_dm: nostr_config.sealed_dm,
        })
    }
}

#[async_trait]
impl Action<'_> for NostrAction<'_> {
    async fn run(&self, params: Option<&MessageParams<'_, '_>>) -> Result<()> {
        let subject = self.message_config.subject(params)?;
        let body = self.message_config.body(params)?;
        let message = format!("{}\n{}", subject, body);

        if self.sealed_dm {
            self.client
                .send_sealed_msg(self.recipient, message, None)
                .await?;
        } else {
            self.client
                .send_direct_msg(self.recipient, message, None)
                .await?;
        }
        Ok(())
    }
}
