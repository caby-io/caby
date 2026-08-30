use std::{io::ErrorKind, path::Path};

use anyhow::{anyhow, Context};
use tokio::fs;
use tracing::{debug, info};

use crate::{
    config::Config,
    controller::{EventHandler, PathLocks, Priority},
    event::{
        Event,
        EventKind::{FileCreated, FileModified, FileMoved, FileRemoved},
    },
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    job::Input,
    share::{cleanup_spec, load_state, reconcile_spec, remove_state, Share, ShareSpec},
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
            let to = &event.path;
            match (
                has_ext(from, CABY_SHARE_SPEC_EXT),
                has_ext(to, CABY_SHARE_SPEC_EXT),
            ) {
                (true, true) => inputs.push((
                    Priority::Interactive,
                    Input::MoveShare {
                        space: event.space.clone(),
                        from: from.clone(),
                        to: to.to_path_buf(),
                    },
                )),
                (false, true) => inputs.extend(reconcile(Priority::Interactive, to)),
                (true, false) => inputs.extend(reconcile(Priority::Background, from)),
                (false, false) => {}
            }
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

pub async fn try_reconcile_share(
    cfg: &Config,
    locks: &PathLocks,
    space_name: &str,
    path: &Path,
    actor: Option<&str>,
) -> Result<()> {
    info!(
        "controller: Starting ReconcileShare on {}/{}",
        space_name,
        path.display()
    );

    let Some(space) = find_space(cfg, space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    reconcile_spec(locks, &space, path, actor).await?;

    Ok(())
}

pub async fn try_move_share(
    cfg: &Config,
    locks: &PathLocks,
    space_name: &str,
    from: &Path,
    to: &Path,
) -> Result<()> {
    info!(
        "controller: Starting MoveShare on {}/{} -> {}",
        space_name,
        from.display(),
        to.display()
    );

    let Some(space) = find_space(cfg, space_name) else {
        return Err(anyhow!("unknown space {}", space_name));
    };

    move_share(locks, &space, from, to).await
}

async fn move_share(locks: &PathLocks, space: &Space, from: &Path, to: &Path) -> Result<()> {
    let (from_guard, to_guard) = locks.acquire_pair(&space.name, from, to).await;
    let live = space.join(SpaceDir::LIVE, to)?;

    let content = match fs::read_to_string(&live).await {
        Ok(content) => content,
        // the moved-to spec is already gone; clean up whatever the move left behind
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return cleanup_spec(space, from, &from_guard).await
        }
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read share spec {:?}", live)))
        }
    };

    let spec = ShareSpec::try_parse(&content)?;
    let existing = match load_state(space, to).await? {
        Some(existing) => Some(existing),
        None => load_state(space, from).await?,
    };
    let share = Share::from_spec(&space.name, to, spec, existing, None)?;
    share.save(space, &to_guard).await?;
    remove_state(space, from).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::PathGuard;

    fn temp_space() -> Space {
        Space {
            name: "rocinante".to_owned(),
            display: "Rocinante".to_owned(),
            path: std::env::temp_dir().join(format!("caby-move-{}", xid::new())),
        }
    }

    async fn seed_share(space: &Space, spec_path: &Path, body: &str) -> Share {
        let spec = ShareSpec::try_parse(body).unwrap();
        let share = Share::from_spec(&space.name, spec_path, spec, None, None).unwrap();
        share.save(space, &PathGuard::test().await).await.unwrap();
        share
    }

    async fn write_live_spec(space: &Space, spec_path: &Path, body: &str) {
        let live = space.join(SpaceDir::LIVE, spec_path).unwrap();
        fs::create_dir_all(live.parent().unwrap()).await.unwrap();
        fs::write(&live, body).await.unwrap();
    }

    #[tokio::test]
    async fn move_carries_the_id_and_drops_the_old_state() {
        let space = temp_space();
        let from = Path::new("photos.share.caby");
        let to = Path::new("albums/trip.share.caby");
        let body = "guest_flows:\n  - permissions: [view]";

        let original = seed_share(&space, from, body).await;
        write_live_spec(&space, to, body).await;

        let locks = PathLocks::default();
        move_share(&locks, &space, from, to).await.unwrap();

        let moved = load_state(&space, to).await.unwrap().unwrap();
        assert_eq!(moved.id, original.id);
        assert_eq!(moved.spec_path, "albums/trip.share.caby");
        assert_eq!(moved.root_entry, "albums");

        assert!(load_state(&space, from).await.unwrap().is_none());
        let via_route = Share::load(&space, &original.id).await.unwrap().unwrap();
        assert_eq!(via_route.spec_path, "albums/trip.share.caby");

        let _ = std::fs::remove_dir_all(&space.path);
    }

    #[tokio::test]
    async fn move_carries_id_when_meta_already_moved_by_ops() {
        let space = temp_space();
        let from = Path::new("photos.share.caby");
        let to = Path::new("albums/trip.share.caby");
        let body = "guest_flows:\n  - permissions: [view]";

        let original = seed_share(&space, from, body).await;

        // simulate ops::rename: the spec AND its meta dir (state) are already at `to`
        write_live_spec(&space, to, body).await;
        let meta_from = space.join(SpaceDir::META, from).unwrap();
        let meta_to = space.join(SpaceDir::META, to).unwrap();
        fs::create_dir_all(meta_to.parent().unwrap()).await.unwrap();
        fs::rename(&meta_from, &meta_to).await.unwrap();

        let locks = PathLocks::default();
        move_share(&locks, &space, from, to).await.unwrap();

        let moved = load_state(&space, to).await.unwrap().unwrap();
        assert_eq!(moved.id, original.id, "id not carried across rename");
        assert_eq!(moved.spec_path, "albums/trip.share.caby");
        assert_eq!(moved.root_entry, "albums");
        let via_route = Share::load(&space, &original.id).await.unwrap().unwrap();
        assert_eq!(via_route.spec_path, "albums/trip.share.caby");

        let _ = std::fs::remove_dir_all(&space.path);
    }

    #[tokio::test]
    async fn cleanup_drops_orphaned_route_when_state_gone() {
        let space = temp_space();
        let spec_path = Path::new("subdir.share.caby");
        let share = seed_share(&space, spec_path, "guest_flows:\n  - permissions: [view]").await;

        let route_file = space.shares().join(format!("{}.json", share.id));
        assert!(route_file.exists());

        // a legacy/malformed route (no spec_path) must not abort the reverse scan
        fs::write(
            space.shares().join("legacyfatroute.json"),
            r#"{"id":"legacyfatroute","owner_id":"x"}"#,
        )
        .await
        .unwrap();

        // simulate ops::remove having deleted the spec's meta dir (the state)
        remove_state(&space, spec_path).await.unwrap();
        assert!(load_state(&space, spec_path).await.unwrap().is_none());

        cleanup_spec(&space, spec_path, &PathGuard::test().await)
            .await
            .unwrap();

        assert!(!route_file.exists(), "orphaned route not dropped");

        let _ = std::fs::remove_dir_all(&space.path);
    }

    #[tokio::test]
    async fn move_to_a_missing_spec_cleans_up_the_source() {
        let space = temp_space();
        let from = Path::new("photos.share.caby");
        let to = Path::new("gone.share.caby");

        let original = seed_share(&space, from, "guest_flows:\n  - permissions: [view]").await;

        let locks = PathLocks::default();
        move_share(&locks, &space, from, to).await.unwrap();

        assert!(load_state(&space, from).await.unwrap().is_none());
        assert!(Share::load(&space, &original.id).await.unwrap().is_none());

        let _ = std::fs::remove_dir_all(&space.path);
    }
}
