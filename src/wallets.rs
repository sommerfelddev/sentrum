use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use bdk::{
    bitcoin::{bip32::ExtendedPubKey, Network, Txid},
    blockchain::{ElectrumBlockchain, GetHeight},
    sled,
    template::{Bip44Public, Bip49Public, Bip84Public, Bip86Public},
    KeychainKind, SyncOptions, TransactionDetails, Wallet,
};
use log::{debug, error, warn};
use serde::Deserialize;

use crate::blockchain::{get_blockchain, ElectrumConfig};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AddressKind {
    Legacy,
    NestedSegwit,
    Segwit,
    Taproot,
}

#[derive(Deserialize, Debug)]
pub struct XpubSpec {
    name: String,
    xpub: String,
    kind: Option<AddressKind>,
}

impl XpubSpec {
    pub fn kind(&self) -> AddressKind {
        self.kind.unwrap_or(AddressKind::Segwit)
    }

    pub fn xpub(&self) -> &str {
        &self.xpub
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug, Hash)]
pub struct DescriptorsSpec {
    name: String,
    primary: String,
    change: Option<String>,
}

impl DescriptorsSpec {
    pub fn get_hash(&self) -> String {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish().to_string()
    }

    pub fn primary(&self) -> &str {
        &self.primary
    }

    pub fn change(&self) -> Option<&String> {
        self.change.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum WalletConfig {
    Xpub(XpubSpec),
    Descriptors(DescriptorsSpec),
}

impl WalletConfig {
    pub fn name(&self) -> &str {
        match self {
            WalletConfig::Xpub(xpub_spec) => xpub_spec.name(),
            WalletConfig::Descriptors(descriptors_spec) => descriptors_spec.name(),
        }
    }
}

fn get_cache_dir(db_name: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or(PathBuf::from("cache"))
        .join(env!("CARGO_PKG_NAME"))
        .join(db_name)
}

fn get_xpub_wallet(xpub_spec: &XpubSpec, network: Network) -> Result<Wallet<sled::Tree>> {
    let xpub: ExtendedPubKey = xpub_spec.xpub().parse().unwrap();
    let fingerprint = xpub.fingerprint();
    let sled = sled::open(get_cache_dir(&fingerprint.to_string()))?.open_tree("wallet")?;
    match xpub_spec.kind() {
        AddressKind::Legacy => Wallet::new(
            Bip44Public(xpub, fingerprint, KeychainKind::External),
            Some(Bip44Public(xpub, fingerprint, KeychainKind::Internal)),
            network,
            sled,
        ),
        AddressKind::NestedSegwit => Wallet::new(
            Bip49Public(xpub, fingerprint, KeychainKind::External),
            Some(Bip49Public(xpub, fingerprint, KeychainKind::Internal)),
            network,
            sled,
        ),
        AddressKind::Segwit => Wallet::new(
            Bip84Public(xpub, fingerprint, KeychainKind::External),
            Some(Bip84Public(xpub, fingerprint, KeychainKind::Internal)),
            network,
            sled,
        ),
        AddressKind::Taproot => Wallet::new(
            Bip86Public(xpub, fingerprint, KeychainKind::External),
            Some(Bip86Public(xpub, fingerprint, KeychainKind::Internal)),
            network,
            sled,
        ),
    }
    .with_context(|| format!("invalid xpub wallet '{}'", xpub))
}

fn get_descriptors_wallet(
    descriptors_spec: &DescriptorsSpec,
    network: Network,
) -> Result<Wallet<sled::Tree>> {
    let sled = sled::open(get_cache_dir(&descriptors_spec.get_hash()))?.open_tree("wallet")?;
    Wallet::new(
        descriptors_spec.primary(),
        descriptors_spec.change().map(String::as_ref),
        network,
        sled,
    )
    .with_context(|| format!("invalid descriptor wallet '{:?}'", descriptors_spec))
}

fn get_wallet(wallet_config: &WalletConfig, network: Network) -> Result<Wallet<sled::Tree>> {
    match &wallet_config {
        WalletConfig::Xpub(xpub_spec) => get_xpub_wallet(xpub_spec, network),
        WalletConfig::Descriptors(descriptors_spec) => {
            get_descriptors_wallet(descriptors_spec, network)
        }
    }
}

pub struct WalletInfo {
    name: String,
    wallet: Wallet<sled::Tree>,
    old_txs: HashSet<Txid>,
    blockchain: ElectrumBlockchain,
}

pub type SafeWalletInfo = Arc<Mutex<WalletInfo>>;

impl WalletInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_height(&self) -> Result<u32, bdk::Error> {
        self.blockchain.get_height()
    }

    pub fn get_network(&self) -> Network {
        self.wallet.network()
    }

    pub fn total_balance(&self) -> Result<u64, bdk::Error> {
        self.wallet.get_balance().map(|b| b.get_total())
    }

    pub fn get_new_txs(&mut self) -> Vec<TransactionDetails> {
        debug!("[{}] syncing wallet", self.name);
        if let Err(e) = self.wallet.sync(&self.blockchain, SyncOptions::default()) {
            warn!("[{}] cannot sync wallet: {:#}", self.name, e);
            return Default::default();
        }
        let tx_list = match self.wallet.list_transactions(false) {
            Ok(txs) => txs,
            Err(e) => {
                warn!("[{}] cannot retrieve transactions: {:#}", self.name, e);
                Default::default()
            }
        };

        let new_txs: Vec<TransactionDetails> = tx_list
            .iter()
            .filter(|&tx| !self.old_txs.contains(&tx.txid))
            .cloned()
            .collect();
        new_txs.iter().for_each(|tx| {
            self.old_txs.insert(tx.txid);
        });
        new_txs
    }
}

pub fn get_wallets(
    wallet_configs: &[WalletConfig],
    electrum_cfg: &ElectrumConfig,
) -> Vec<SafeWalletInfo> {
    let mut result: Vec<SafeWalletInfo> = vec![];
    for wallet_config in wallet_configs.iter() {
        let name = wallet_config.name();
        match get_wallet(wallet_config, electrum_cfg.network()) {
            Ok(w) => {
                result.push(Arc::new(Mutex::new(WalletInfo {
                    name: name.to_string(),
                    wallet: w,
                    old_txs: Default::default(),
                    blockchain: get_blockchain(electrum_cfg).unwrap(),
                })));
            }
            Err(e) => {
                error!("[{}] cannot setup wallet: {:#}", name, e);
            }
        }
    }
    result
}
