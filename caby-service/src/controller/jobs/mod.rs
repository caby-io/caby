use anyhow::anyhow;
use tokio::time;

use crate::{
    config::Config,
    job::{Input, Job},
    Result,
};

use super::EventHandler;

pub mod shares;

pub use shares::{try_move_share, try_reconcile_share, try_scan_shares};

pub fn handlers() -> Vec<EventHandler> {
    shares::handlers()
}

async fn dispatch(cfg: &Config, input: &Input, actor: Option<&str>) -> Result<()> {
    match input {
        Input::ScanShares { space } => try_scan_shares(cfg, space).await,
        Input::ReconcileShare { space, path } => try_reconcile_share(cfg, space, path, actor).await,
        Input::MoveShare { space, from, to } => try_move_share(cfg, space, from, to).await,
    }
}

pub async fn run(cfg: &Config, job: &Job) -> Result<()> {
    let input = &job.input;
    let timeout = input.timeout();
    match time::timeout(timeout, dispatch(cfg, input, job.actor.as_deref())).await {
        Ok(result) => result,
        Err(_elapsed) => Err(anyhow!(
            "job {:?} exceeded its {:?} timeout",
            input,
            timeout
        )),
    }
}
