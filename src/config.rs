use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use const_format::{formatcp, map_ascii_case, Case};
use log::info;
use serde::Deserialize;

use crate::{
    actions::AnyActionConfig, blockchain::ElectrumConfig, message::MessageConfig,
    wallets::WalletConfig,
};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Path to toml configuration file
    config: Option<String>,
    /// Perform configured actions on a test notification
    #[arg(short, long)]
    test: bool,
    /// Notify for every past transaction (careful: if you have a long transaction history, this
    /// can SPAM your configured actions
    #[arg(short, long)]
    notify_past_txs: bool,
}

impl Args {
    pub fn config(&self) -> Option<&str> {
        self.config.as_deref()
    }

    pub fn test(&self) -> bool {
        self.test
    }

    pub fn notify_past_txs(&self) -> bool {
        self.notify_past_txs
    }
}

fn get_config_filename() -> &'static str {
    formatcp!("{}.toml", env!("CARGO_PKG_NAME"))
}

fn get_config_env_var() -> &'static str {
    formatcp!(
        "{}_CONFIG",
        map_ascii_case!(Case::Upper, env!("CARGO_PKG_NAME"))
    )
}

fn get_cwd_config_path() -> PathBuf {
    PathBuf::from(".").join(get_config_filename())
}

fn get_config_path_impl(user_config_dir: &Path) -> PathBuf {
    user_config_dir
        .join(env!("CARGO_PKG_NAME"))
        .join(get_config_filename())
}

fn get_user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| get_config_path_impl(&p))
}

fn get_system_config_path() -> PathBuf {
    get_config_path_impl(&systemd_directories::config_dir().unwrap_or(PathBuf::from("/etc")))
}

fn get_config_path(maybe_arg_config: &Option<&str>) -> Result<PathBuf> {
    if let Some(arg_path) = maybe_arg_config {
        return Ok(PathBuf::from(arg_path));
    }

    if let Ok(env_path) = env::var(get_config_env_var()) {
        return Ok(PathBuf::from(env_path));
    }

    let cwd_config_path = get_cwd_config_path();
    if cwd_config_path.try_exists().is_ok_and(|x| x) {
        return Ok(cwd_config_path);
    }

    if let Some(user_config_path) = get_user_config_path() {
        if user_config_path.try_exists().is_ok_and(|x| x) {
            return Ok(user_config_path);
        }
    }

    let system_config_path = get_system_config_path();
    if system_config_path.try_exists().is_ok_and(|x| x) {
        return Ok(system_config_path);
    }

    bail!(
        "no configuration file was passed as first argument, nor by the '{}' environment variable, nor did one exist in the default search paths: '{}', '{}', '{}'",
        get_config_env_var(),
        get_cwd_config_path().display(),
        get_user_config_path().unwrap_or_default().display(),
        get_system_config_path().display()
    );
}

#[derive(Deserialize, Debug)]
pub struct Config {
    wallets: Vec<WalletConfig>,
    #[serde(default)]
    electrum: ElectrumConfig,
    #[serde(default)]
    message: MessageConfig,
    #[serde(default)]
    actions: Vec<AnyActionConfig>,
}

impl Config {
    pub fn electrum(&self) -> &ElectrumConfig {
        &self.electrum
    }

    pub fn wallets(&self) -> &[WalletConfig] {
        &self.wallets
    }

    pub fn message(&self) -> &MessageConfig {
        &self.message
    }

    pub fn actions(&self) -> &[AnyActionConfig] {
        &self.actions
    }
}

pub fn get_config(maybe_arg_config: &Option<&str>) -> Result<Config> {
    let config_path = get_config_path(maybe_arg_config)?;
    info!("reading configuration from '{}'", config_path.display());
    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("could not read config file '{}'", config_path.display()))?;
    toml::from_str(&config_content)
        .with_context(|| format!("could not parse config file '{}'", config_path.display(),))
}
