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
    share::{load_state, Share, ShareSpec},
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

    let content = match fs::read_to_string(&live).await {
        Ok(content) => content,
        // share spec doesn't exist, attempt an opportunistic cleanup of the state/route
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return try_cleanup_share(cfg, &space, path).await
        }
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read share spec {:?}", live)))
        }
    };

    let spec = ShareSpec::try_parse(&content)?;
    let existing = load_state(&space, path).await?;
    let share = Share::from_spec(&space.name, path, spec, existing)?;
    share.save(&space).await?;

    Ok(())
}

pub async fn try_cleanup_share(_cfg: &Config, space: &Space, path: &Path) -> Result<()> {
    let Some(share) = load_state(space, path).await? else {
        info!(
            "controller: no share state to clean up for {}/{}",
            space.name,
            path.display()
        );
        return Ok(());
    };

    Share::delete(space, &share.id).await
}
