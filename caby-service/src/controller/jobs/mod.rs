use crate::{
    config::Config,
    controller::{scan, PathLocks},
    job::{Input, Job},
    Result,
};

use super::EventHandler;

pub mod shares;

use anyhow::anyhow;
pub use shares::{try_move_share, try_reconcile_share};
use tokio::time;

pub fn handlers() -> Vec<EventHandler> {
    shares::handlers()
}

async fn dispatch(
    cfg: &Config,
    locks: &PathLocks,
    input: &Input,
    actor: Option<&str>,
) -> Result<()> {
    match input {
        Input::ScanSpace { space } => scan::try_scan_space(cfg, locks, space).await,
        Input::ReconcileShare { space, path } => {
            try_reconcile_share(cfg, locks, space, path, actor).await
        }
        Input::MoveShare { space, from, to } => try_move_share(cfg, locks, space, from, to).await,
    }
}

pub async fn run(cfg: &Config, locks: &PathLocks, job: &Job) -> Result<()> {
    let input = &job.input;
    let timeout = input.timeout();
    match time::timeout(timeout, dispatch(cfg, locks, input, job.actor.as_deref())).await {
        Ok(result) => result,
        Err(_elapsed) => Err(anyhow!(
            "job {:?} exceeded its {:?} timeout",
            input,
            timeout
        )),
    }
}
