use anyhow::{Context, Result};
use bdk::{
    bitcoin::Network,
    blockchain::{ElectrumBlockchain, GetHeight},
    electrum_client::{Client, ConfigBuilder, Socks5Config},
};
use log::{debug, info, warn};
use serde::Deserialize;

fn get_default_electrum_server(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "ssl://fulcrum.sethforprivacy.com:50002",
        Network::Testnet => "ssl://electrum.blockstream.info:60002",
        Network::Signet => "ssl://mempool.space:60602",
        _ => panic!("unsupported network"),
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct ElectrumConfig {
    url: Option<String>,

    network: Option<Network>,

    socks5: Option<String>,

    #[serde(default)]
    certificate_validation: bool,
}

impl ElectrumConfig {
    pub fn url(&self) -> &str {
        self.url
            .as_deref()
            .unwrap_or(get_default_electrum_server(self.network()))
    }

    pub fn network(&self) -> Network {
        self.network.unwrap_or(Network::Bitcoin)
    }

    pub fn certificate_validation(&self) -> bool {
        self.certificate_validation
    }

    pub fn socks5(&self) -> Option<Socks5Config> {
        self.socks5.as_ref().map(Socks5Config::new)
    }
}

pub fn get_blockchain(electrum_cfg: &ElectrumConfig) -> Result<ElectrumBlockchain> {
    let server_cfg = ConfigBuilder::new()
        .validate_domain(electrum_cfg.certificate_validation())
        .socks5(electrum_cfg.socks5())
        .build();
    let electrum_url = electrum_cfg.url();
    let client = Client::from_config(electrum_url, server_cfg)
        .with_context(|| "could not configure electrum client".to_string())?;
    Ok(ElectrumBlockchain::from(client))
}

pub struct BlockchainState {
    height: Option<u32>,
    url: String,
    blockchain: ElectrumBlockchain,
}

impl BlockchainState {
    pub fn new(electrum_cfg: &ElectrumConfig) -> Result<Self> {
        Ok(Self {
            height: Default::default(),
            url: String::from(electrum_cfg.url()),
            blockchain: get_blockchain(electrum_cfg)?,
        })
    }

    pub fn update_height(&mut self) {
        match self.blockchain.get_height() {
            Ok(polled_height) => {
                match self.height {
                    Some(h) => {
                        if polled_height != h {
                            self.height = Some(polled_height);
                            debug!("current block height: {}", polled_height);
                        }
                    }
                    None => {
                        self.height = Some(polled_height);
                        info!("connected to '{}'", self.url);
                        info!("current block height: {}", polled_height);
                    }
                };
            }
            Err(e) => warn!("could not reach '{}': {}", self.url, e),
        };
    }
}
