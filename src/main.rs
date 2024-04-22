extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::process::exit;
use std::time::Duration;

use actions::Action;
use async_scoped::TokioScope;
use clap::Parser;
use human_panic::setup_panic;

use anyhow::{bail, Context, Result};
#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::sleep;

mod actions;
mod blockchain;
mod config;
mod message;
mod wallets;

use crate::actions::get_actions;
use crate::message::MessageParams;
use crate::{
    blockchain::BlockchainState,
    config::{get_config, Args},
    wallets::{get_wallets, SafeWalletInfo},
};

fn set_logger() {
    pretty_env_logger::formatted_builder()
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Info)
        .parse_default_env()
        .init();
}

fn set_signal_handlers() -> Result<()> {
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            return e;
        }
        warn!("received ctrl-c signal. Exiting...");
        exit(0)
    });
    #[cfg(unix)]
    tokio::spawn(async move {
        let mut stream = match signal(SignalKind::terminate()) {
            Err(e) => return e,
            Ok(s) => s,
        };
        stream.recv().await;
        warn!("received process termination signal. Exiting...");
        exit(0)
    });
    #[cfg(unix)]
    tokio::spawn(async move {
        let mut stream = match signal(SignalKind::hangup()) {
            Err(e) => return e,
            Ok(s) => s,
        };
        stream.recv().await;
        warn!("received process hangup signal. Exiting...");
        exit(0)
    });
    Ok(())
}

async fn run_test_actions(actions: &[&(dyn Action<'_> + Sync)]) {
    TokioScope::scope_and_block(|s| {
        for &action in actions {
            s.spawn(action.run(None));
        }
    });
}

fn get_and_handle_new_txs(
    wallet_info: &SafeWalletInfo,
    actions: &[&(dyn Action<'_> + Sync)],
) -> Result<()> {
    let mut locked_wallet_info = wallet_info.lock().unwrap();
    let txs = locked_wallet_info.get_new_txs();
    TokioScope::scope_and_block(|s| {
        for tx in txs.iter() {
            let params = MessageParams::new(tx, &locked_wallet_info);
            s.spawn(async move {
                TokioScope::scope_and_block(|s| {
                    for &action in actions {
                        s.spawn(action.run(Some(&params)));
                    }
                });
            });
        }
    });
    Ok(())
}

async fn update_blockchain_thread(blockchain_state: &mut BlockchainState) {
    loop {
        blockchain_state.update_height();
        sleep(Duration::from_secs(60)).await;
    }
}

async fn watch_wallet_thread(wallet_info: &SafeWalletInfo, actions: &[&(dyn Action<'_> + Sync)]) {
    loop {
        if let Err(e) = get_and_handle_new_txs(wallet_info, actions) {
            warn!("{:?}", e);
        }
    }
}

async fn initial_wallet_sync(blockchain_state: &mut BlockchainState, wallets: &[SafeWalletInfo]) {
    TokioScope::scope_and_block(|s| {
        s.spawn(async { blockchain_state.update_height() });
        for wallet_info in wallets {
            s.spawn(async {
                if let Err(e) = get_and_handle_new_txs(wallet_info, &[]) {
                    warn!("{:?}", e);
                }
            });
        }
    });
}

async fn watch_wallets(
    blockchain_state: &mut BlockchainState,
    wallets: &[SafeWalletInfo],
    actions: &[&(dyn Action<'_> + Sync)],
) {
    TokioScope::scope_and_block(|s| {
        s.spawn(update_blockchain_thread(blockchain_state));
        for wallet_info in wallets {
            s.spawn(watch_wallet_thread(wallet_info, actions));
        }
    });
}

async fn do_main() -> Result<()> {
    setup_panic!();
    let args = Args::parse();
    set_logger();
    set_signal_handlers().context("failed to setup a signal termination handler")?;

    let config = get_config(&args.config())?;

    let actions = get_actions(config.message(), config.actions()).await;
    if actions.is_empty() {
        bail!("no actions properly configured");
    }
    let actions_ref = actions.iter().map(Box::as_ref).collect::<Vec<_>>();

    if args.test() {
        run_test_actions(&actions_ref).await;
        return Ok(());
    }

    let mut blockchain_state = BlockchainState::new(config.electrum())?;

    let wallets = get_wallets(config.wallets(), config.electrum());
    if wallets.is_empty() {
        bail!("no wallets properly configured");
    }

    if !args.notify_past_txs() {
        info!("initial wallet sync");
        initial_wallet_sync(&mut blockchain_state, &wallets).await;
    }
    info!("listening for new relevant events");
    watch_wallets(&mut blockchain_state, &wallets, &actions_ref).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = do_main().await {
        error!("{:?}", e);
        exit(1);
    }
}
