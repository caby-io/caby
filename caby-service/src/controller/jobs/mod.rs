use anyhow::anyhow;
use tokio::time;

use crate::{config::Config, job::Input, Result};

use super::EventHandler;

pub mod shares;

pub use shares::{try_reconcile_share, try_scan_shares};

pub fn handlers() -> Vec<EventHandler> {
    shares::handlers()
}

async fn dispatch(cfg: &Config, input: &Input) -> Result<()> {
    match input {
        Input::ScanShares { space } => try_scan_shares(cfg, space).await,
        Input::ReconcileShare { space, path } => try_reconcile_share(cfg, space, path).await,
    }
}

pub async fn run(cfg: &Config, input: &Input) -> Result<()> {
    let timeout = input.timeout();
    match time::timeout(timeout, dispatch(cfg, input)).await {
        Ok(result) => result,
        Err(_elapsed) => Err(anyhow!(
            "job {:?} exceeded its {:?} timeout",
            input,
            timeout
        )),
    }
}
