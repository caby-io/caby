use std::{io::ErrorKind, path::Path};

use anyhow::{anyhow, Context};
use tokio::fs;
use tracing::{debug, info};

use crate::{
    config::Config,
    controller::{EventHandler, Priority},
    event::{
        Event,
        EventKind::{FileCreated, FileModified, FileMoved, FileRemoved},
    },
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    job::Input,
    share::CABY_SHARE_STATE_FILE,
    space::{Space, SpaceDir},
    Result,
};

fn handle_event(event: &Event) -> Vec<(Priority, Input)> {
    let reconcile = |priority, path: &Path| {
        if !has_ext(path, CABY_SHARE_SPEC_EXT) {
            return None;
        }
        Some((
            priority,
            Input::ReconcileShare {
                space: event.space.clone(),
                path: path.to_path_buf(),
            },
        ))
    };

    let mut inputs = vec![];

    match &event.kind {
        FileCreated | FileModified => {
            inputs.extend(reconcile(Priority::Interactive, &event.path));
        }
        FileMoved { from } => {
            inputs.extend(reconcile(Priority::Interactive, &event.path));
            inputs.extend(reconcile(Priority::Background, from));
        }
        FileRemoved => {
            inputs.extend(reconcile(Priority::Background, &event.path));
        }
    }

    inputs
}

pub fn handlers() -> Vec<EventHandler> {
    vec![handle_event]
}

pub async fn try_scan_shares(cfg: &Config, space: &str) -> Result<()> {
    debug!("controller: ScanShares {} is not wired up yet", space);
    Ok(())
}

fn find_space(cfg: &Config, name: &str) -> Option<Space> {
    cfg.runtime.load().spaces.get(name).map(Space::from)
}

pub async fn try_reconcile_share(cfg: &Config, space_name: &str, path: &Path) -> Result<()> {
    info!(
        "controller: Starting ReconcileShare on {}/{}",
        space_name,
        path.display()
    );

    let Some(space) = find_space(cfg, space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    let live = space.join(SpaceDir::LIVE, path)?;

    let spec = match fs::read_to_string(&live).await {
        Ok(spec) => spec,
        // share spec doesn't exist, attempt an opportunistic cleanup of the state/route
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return try_cleanup_share(cfg, &space, path).await
        }
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read share spec {:?}", live)))
        }
    };

    // todo: sync the state and route to the spec
    Ok(())
}

pub async fn try_cleanup_share(cfg: &Config, space: &Space, path: &Path) -> Result<()> {
    let state_path = space.join(SpaceDir::META, &path.join(CABY_SHARE_STATE_FILE))?;

    let state = match fs::read(&state_path).await {
        Ok(s) => s,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            info!(
                "controller: could not find share state for {}/{}",
                space.name,
                path.display()
            );
            return Ok(());
        }
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read share state {:?}", state_path)))
        }
    };

    Ok(())
}
