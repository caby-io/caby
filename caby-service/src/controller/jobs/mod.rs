use crate::{
    config::Config,
    controller::{scan, PathLocks, Priority},
    job::{Input, Job},
    Result,
};

use super::EventHandler;

pub mod shares;
pub mod thumbs;

use anyhow::anyhow;
pub use shares::{try_move_share, try_reconcile_share};
pub use thumbs::{try_generate_dir_thumbs, try_generate_file_thumb};
use tokio::time;

pub fn handlers() -> Vec<EventHandler> {
    shares::handlers()
        .into_iter()
        .chain(thumbs::handlers())
        .collect()
}

async fn dispatch(
    cfg: &Config,
    locks: &PathLocks,
    input: &Input,
    actor: Option<&str>,
) -> Result<Vec<(Priority, Input)>> {
    match input {
        Input::ScanSpace { space } => scan::try_scan_space(cfg, locks, space).await,
        Input::ReconcileShare { space, path } => {
            try_reconcile_share(cfg, locks, space, path, actor)
                .await
                .map(|()| vec![])
        }
        Input::MoveShare { space, from, to } => try_move_share(cfg, locks, space, from, to)
            .await
            .map(|()| vec![]),
        Input::GenerateFileThumb { space, path } => try_generate_file_thumb(cfg, space, path)
            .await
            .map(|()| vec![]),
        Input::GenerateDirThumbs {
            space,
            dir,
            recursive,
            after,
        } => try_generate_dir_thumbs(cfg, space, dir, *recursive, after.as_deref()).await,
    }
}

pub async fn run(cfg: &Config, locks: &PathLocks, job: &Job) -> Result<Vec<(Priority, Input)>> {
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
