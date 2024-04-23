extern crate chrono;
extern crate strfmt;

use anyhow::{bail, Context, Result};
use bdk::{bitcoin::Network, TransactionDetails};
use chrono::DateTime;
use serde::Deserialize;
use strfmt::strfmt;

use crate::wallets::WalletInfo;

pub struct MessageParams<'a, 'b> {
    tx: &'a TransactionDetails,
    wallet: &'b str,
    total_balance: u64,
    current_height: u32,
    network: Network,
}

impl<'a, 'b> MessageParams<'a, 'b> {
    pub fn new(tx: &'a TransactionDetails, wallet: &'b WalletInfo) -> Self {
        Self {
            tx,
            wallet: wallet.name(),
            total_balance: wallet.total_balance().unwrap_or_default(),
            current_height: wallet.get_height().unwrap_or_default(),
            network: wallet.get_network(),
        }
    }

    pub fn tx_net(&self) -> i64 {
        (self.tx.received as i64) - (self.tx.sent as i64)
    }

    pub fn tx_height(&self) -> Option<u32> {
        self.tx.confirmation_time.as_ref().map(|x| x.height)
    }

    pub fn confs(&self) -> u32 {
        let current_height = self.current_height;
        self.tx_height()
            .map(|h| {
                if current_height >= h {
                    current_height - h
                } else {
                    0
                }
            })
            .unwrap_or_default()
    }

    pub fn conf_timestamp(&self) -> String {
        self.tx
            .confirmation_time
            .as_ref()
            .map(|x| {
                DateTime::from_timestamp(x.timestamp as i64, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string()
            })
            .unwrap_or_default()
    }

    pub fn txid(&self) -> String {
        self.tx.txid.to_string()
    }
    pub fn txid_short(&self) -> String {
        let txid = self.txid();
        format!("{}...{}", &txid[..6], &txid[txid.len() - 6..])
    }

    fn tx(&self) -> &TransactionDetails {
        self.tx
    }

    pub fn network(&self) -> Network {
        self.network
    }
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormat {
    Plain,
    Markdown,
    Html,
}

#[derive(Deserialize, Default, Debug)]
pub struct BlockExplorers {
    mainnet: Option<String>,
    testnet: Option<String>,
    signet: Option<String>,
}

impl BlockExplorers {
    fn mainnet(&self) -> &str {
        self.mainnet
            .as_deref()
            .unwrap_or("https://mempool.space/tx/{txid}")
    }

    fn testnet(&self) -> &str {
        self.testnet
            .as_deref()
            .unwrap_or("https://mempool.space/testnet/tx/{txid}")
    }

    fn signet(&self) -> &str {
        self.signet
            .as_deref()
            .unwrap_or("https://mempool.space/signet/tx/{txid}")
    }

    pub fn get_tx_url_template(&self, network: &Network) -> Result<&str> {
        Ok(match network {
            Network::Bitcoin => self.mainnet(),
            Network::Testnet => self.testnet(),
            Network::Signet => self.signet(),
            _ => bail!("unsupported network"),
        })
    }
    pub fn get_tx_url(&self, network: &Network, txid: &str) -> Result<String> {
        let template = self.get_tx_url_template(network)?;
        strfmt!(template, txid => txid.to_string())
            .with_context(|| format!("bad block explorer URL template '{}'", template))
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct MessageConfig {
    subject: Option<String>,
    body: Option<String>,
    format: Option<MessageFormat>,
    #[serde(default)]
    block_explorers: BlockExplorers,
}

impl MessageConfig {
    pub fn subject_template(&self) -> &str {
        self.subject
            .as_deref()
            .unwrap_or("[{wallet}] new transaction")
    }

    pub fn body_template(&self) -> &str {
        self.body
            .as_deref()
            .unwrap_or("net: {tx_net} sats, balance: {total_balance} sats, txid: {txid_short}")
    }

    pub fn replace_template_params(
        &self,
        template: &str,
        params: &MessageParams,
    ) -> Result<String> {
        strfmt!(template,
            tx_net => params.tx_net(),
            wallet => params.wallet.to_string(),
            total_balance => params.total_balance,
            txid => params.txid(),
            txid_short => params.txid_short(),
            received => params.tx().received,
            sent => params.tx().sent,
            fee => params.tx().fee.unwrap_or_default(),
            current_height => params.current_height,
            tx_height => params.tx_height().unwrap_or_default(),
            confs => params.confs(),
            conf_timestamp => params.conf_timestamp(),
            tx_url => self.get_tx_url(Some(params))?
        )
        .with_context(|| format!("invalid template '{}'", template))
    }

    pub fn subject(&self, params: Option<&MessageParams>) -> Result<String> {
        match params {
            Some(p) => self.replace_template_params(self.subject_template(), p),
            None => Ok(self.subject_template().to_string()),
        }
    }

    pub fn body(&self, params: Option<&MessageParams>) -> Result<String> {
        match params {
            Some(p) => self.replace_template_params(self.body_template(), p),
            None => Ok(self.body_template().to_string()),
        }
    }

    #[allow(dead_code)]
    pub fn format(&self) -> &MessageFormat {
        self.format.as_ref().unwrap_or(&MessageFormat::Plain)
    }

    pub fn get_tx_url(&self, params: Option<&MessageParams>) -> Result<String> {
        match params {
            Some(p) => self.block_explorers.get_tx_url(&p.network(), &p.txid()),
            None => Ok(self
                .block_explorers
                .get_tx_url_template(&Network::Bitcoin)?
                .to_string()),
        }
    }
}
