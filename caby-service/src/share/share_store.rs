use std::{
    collections::BTreeMap,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use jiff::Timestamp;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    config::Config,
    controller::{PathGuard, PathLocks},
    space::{Space, SpaceDir},
    Result,
};

use super::{Grant, Share, ShareAccessFlow, ShareSpec, CABY_SHARE_STATE_FILE};

#[derive(Serialize, Deserialize)]
struct ShareStateFile {
    id: String,
    owner_id: String,
    space: String,
    spec_path: String,
    root_entry: String,

    #[serde(default)]
    account_allowlist: BTreeMap<String, Grant>,
    #[serde(default)]
    guest_allowlist: BTreeMap<String, Grant>,

    #[serde(default)]
    account_flows: Vec<ShareAccessFlow>,
    #[serde(default)]
    guest_flows: Vec<ShareAccessFlow>,

    created_at: Timestamp,
    expires_at: Option<Timestamp>,
}

impl From<ShareStateFile> for Share {
    fn from(stored: ShareStateFile) -> Self {
        Share {
            id: stored.id,
            owner_id: stored.owner_id,
            space: stored.space,
            spec_path: stored.spec_path,
            root_entry: stored.root_entry,
            account_allowlist: stored.account_allowlist.into_iter().collect(),
            guest_allowlist: stored.guest_allowlist.into_iter().collect(),
            account_flows: stored.account_flows,
            guest_flows: stored.guest_flows,
            created_at: stored.created_at,
            expires_at: stored.expires_at,
        }
    }
}

impl From<&Share> for ShareStateFile {
    fn from(share: &Share) -> Self {
        ShareStateFile {
            id: share.id.clone(),
            owner_id: share.owner_id.clone(),
            space: share.space.clone(),
            spec_path: share.spec_path.clone(),
            root_entry: share.root_entry.clone(),
            account_allowlist: share
                .account_allowlist
                .iter()
                .map(|(id, grant)| (id.clone(), grant.clone()))
                .collect(),
            guest_allowlist: share
                .guest_allowlist
                .iter()
                .map(|(id, grant)| (id.clone(), grant.clone()))
                .collect(),
            account_flows: share.account_flows.clone(),
            guest_flows: share.guest_flows.clone(),
            created_at: share.created_at,
            expires_at: share.expires_at,
        }
    }
}

fn route_path(shares_root: &Path, id: &str) -> Result<PathBuf> {
    let path = shares_root.join(format!("{id}.json")).clean();
    if !path.starts_with(shares_root) {
        return Err(anyhow!("share id out of bounds: {:?}", id));
    }
    Ok(path)
}

fn state_path(space: &Space, spec_path: &Path) -> Result<PathBuf> {
    space.join(SpaceDir::META, &spec_path.join(CABY_SHARE_STATE_FILE))
}

async fn load_share_file(path: &Path) -> Result<Share> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("could not read share file {:?}", path))?;
    let stored: ShareStateFile = serde_json::from_str(&content)
        .with_context(|| format!("could not parse share file {:?}", path))?;

    Ok(Share::from(stored))
}

pub async fn get_shares_in_space(shares_root: &Path, space: &Space) -> Result<Vec<Share>> {
    let mut read_dir = match fs::read_dir(shares_root).await {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read shares dir {:?}", shares_root)))
        }
    };

    let mut shares = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .with_context(|| format!("could not read shares dir {:?}", shares_root))?
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        match read_route(shares_root, id).await {
            Ok(Some(route)) if route.space == space.name => {
                match load_state(space, Path::new(&route.spec_path)).await {
                    Ok(Some(share)) => shares.push(share),
                    Ok(None) => warn!("skipping dangling share route {:?}", path),
                    Err(err) => warn!("skipping unreadable share state {:?}: {:#}", path, err),
                }
            }
            Ok(_) => {}
            Err(err) => warn!("skipping unreadable share route {:?}: {:#}", path, err),
        }
    }

    Ok(shares)
}

impl Share {
    async fn write_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("could not create dir {:?}", parent))?;
        }

        let stored = ShareStateFile::from(self);
        let serialized =
            serde_json::to_string_pretty(&stored).context("could not serialize share")?;
        fs::write(path, serialized)
            .await
            .with_context(|| format!("could not write share file {:?}", path))?;

        Ok(())
    }

    pub async fn save(&self, shares_root: &Path, space: &Space, _guard: &PathGuard) -> Result<()> {
        let spec_path = Path::new(&self.spec_path);
        self.write_to(&state_path(space, spec_path)?).await?;
        write_route(shares_root, &self.id, &self.space, &self.spec_path).await
    }

    pub async fn resolve(cfg: &Config, id: &str) -> Result<Option<(Space, Share)>> {
        let Some(route) = read_route(&cfg.shares_path, id).await? else {
            return Ok(None);
        };
        let Some(space) = cfg.runtime.load().spaces.get(&route.space).map(Space::from) else {
            warn!("share {} routes to unknown space {}", id, route.space);
            return Ok(None);
        };
        let Some(share) = load_state(&space, Path::new(&route.spec_path)).await? else {
            return Ok(None);
        };
        Ok(Some((space, share)))
    }

    pub async fn delete(
        shares_root: &Path,
        space: &Space,
        id: &str,
        _guard: &PathGuard,
    ) -> Result<()> {
        let Some(route) = read_route(shares_root, id).await? else {
            return Ok(());
        };
        let spec_path = Path::new(&route.spec_path);

        remove_spec(space, spec_path).await?;
        remove_state(space, spec_path).await?;
        remove_route(shares_root, id).await
    }
}

pub async fn reconcile_spec(
    shares_root: &Path,
    locks: &PathLocks,
    space: &Space,
    spec_path: &Path,
    actor: Option<&str>,
) -> Result<Option<Share>> {
    let guard = locks.acquire(&space.name, spec_path).await;
    let live = space.join(SpaceDir::LIVE, spec_path)?;

    let content = match fs::read_to_string(&live).await {
        Ok(content) => content,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            cleanup_spec(shares_root, space, spec_path, &guard).await?;
            return Ok(None);
        }
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read share spec {:?}", live)))
        }
    };

    let spec = ShareSpec::try_parse(&content)?;
    let existing = load_state(space, spec_path).await?;
    let share = Share::from_spec(&space.name, spec_path, spec, existing, actor)?;
    share.save(shares_root, space, &guard).await?;

    Ok(Some(share))
}

pub async fn cleanup_spec(
    shares_root: &Path,
    space: &Space,
    spec_path: &Path,
    guard: &PathGuard,
) -> Result<()> {
    if let Some(share) = load_state(space, spec_path).await? {
        return Share::delete(shares_root, space, &share.id, guard).await;
    }
    remove_routes_for_spec(shares_root, &space.name, spec_path).await
}

pub async fn load_state(space: &Space, spec_path: &Path) -> Result<Option<Share>> {
    let path = state_path(space, spec_path)?;
    if !fs::try_exists(&path)
        .await
        .with_context(|| format!("could not check share state {:?}", path))?
    {
        return Ok(None);
    }

    Ok(Some(load_share_file(&path).await?))
}

pub async fn remove_state(space: &Space, spec_path: &Path) -> Result<()> {
    let path = state_path(space, spec_path)?;
    match fs::remove_file(&path).await {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not remove share state {:?}", path)))
        }
    }

    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent).await;
    }

    Ok(())
}

async fn remove_spec(space: &Space, spec_path: &Path) -> Result<()> {
    let live = space.join(SpaceDir::LIVE, spec_path)?;
    match fs::remove_file(&live).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow!(err).context(format!("could not remove share spec {:?}", live))),
    }
}

#[derive(Serialize, Deserialize)]
struct RouteFile {
    space: String,
    spec_path: String,
}

async fn read_route(shares_root: &Path, id: &str) -> Result<Option<RouteFile>> {
    let path = route_path(shares_root, id)?;
    match fs::read_to_string(&path).await {
        Ok(content) => {
            let route: RouteFile = serde_json::from_str(&content)
                .with_context(|| format!("could not parse share route {:?}", path))?;
            Ok(Some(route))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(anyhow!(err).context(format!("could not read share route {:?}", path))),
    }
}

async fn write_route(shares_root: &Path, id: &str, space: &str, spec_path: &str) -> Result<()> {
    let path = route_path(shares_root, id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("could not create shares dir {:?}", parent))?;
    }

    let serialized = serde_json::to_string_pretty(&RouteFile {
        space: space.to_owned(),
        spec_path: spec_path.to_owned(),
    })
    .context("could not serialize share route")?;
    fs::write(&path, serialized)
        .await
        .with_context(|| format!("could not write share route {:?}", path))?;

    Ok(())
}

async fn remove_route(shares_root: &Path, id: &str) -> Result<()> {
    let path = route_path(shares_root, id)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow!(err).context(format!("could not remove share route {:?}", path))),
    }
}

pub async fn remove_routes_for_spec(
    shares_root: &Path,
    space: &str,
    spec_path: &Path,
) -> Result<()> {
    let mut read_dir = match fs::read_dir(shares_root).await {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read shares dir {:?}", shares_root)))
        }
    };

    let target = spec_path.to_string_lossy();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .with_context(|| format!("could not read shares dir {:?}", shares_root))?
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        match read_route(shares_root, id).await {
            Ok(Some(route)) if route.space == space && route.spec_path == target => {
                remove_route(shares_root, id).await?
            }
            Ok(_) => {}
            Err(err) => warn!("skipping unreadable share route {:?}: {:#}", path, err),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        share::{ShareAuth, ShareLimits},
        user::Permission,
    };
    use std::collections::BTreeSet;

    fn sample(account_flows: Vec<ShareAccessFlow>, guest_flows: Vec<ShareAccessFlow>) -> Share {
        Share::new(
            "holden",
            "rocinante",
            "photos/public.share.caby",
            "photos",
            account_flows,
            guest_flows,
            None,
        )
    }

    fn temp_space() -> Space {
        Space {
            name: "rocinante".to_owned(),
            display: "Rocinante".to_owned(),
            path: std::env::temp_dir().join(format!("caby-share-{}", xid::new())),
        }
    }

    fn shares_root(space: &Space) -> PathBuf {
        space.path.join("shares")
    }

    fn cleanup(space: &Space) {
        let _ = std::fs::remove_dir_all(&space.path);
    }

    #[tokio::test]
    async fn save_load_round_trip() {
        let space = temp_space();
        let share = Share::new(
            "holden",
            &space.name,
            "photos.share.caby",
            "photos",
            vec![],
            vec![ShareAccessFlow {
                auth: ShareAuth::Open,
                permissions: BTreeSet::from([Permission::View, Permission::Download]),
                limits: Some(ShareLimits {
                    max_file_bytes: Some(1024),
                    max_bytes_per_day: None,
                    max_files_per_day: Some(10),
                }),
            }],
            None,
        );

        let root = shares_root(&space);
        share
            .save(&root, &space, &PathGuard::test().await)
            .await
            .unwrap();

        let route = read_route(&root, &share.id).await.unwrap().unwrap();
        assert_eq!(route.space, space.name);
        let loaded = load_state(&space, Path::new(&route.spec_path))
            .await
            .unwrap();
        assert_eq!(loaded, Some(share));

        cleanup(&space);
    }

    #[tokio::test]
    async fn share_route_round_trip() {
        let space = temp_space();
        let root = shares_root(&space);

        assert!(read_route(&root, "abc123").await.unwrap().is_none());

        write_route(&root, "abc123", &space.name, "photos/trip.share.caby")
            .await
            .unwrap();
        let route = read_route(&root, "abc123").await.unwrap().unwrap();
        assert_eq!(route.space, space.name);
        assert_eq!(route.spec_path, "photos/trip.share.caby");

        remove_route(&root, "abc123").await.unwrap();
        assert!(read_route(&root, "abc123").await.unwrap().is_none());

        cleanup(&space);
    }

    #[tokio::test]
    async fn load_missing_is_none() {
        let space = temp_space();
        let root = shares_root(&space);
        assert!(read_route(&root, "does-not-exist").await.unwrap().is_none());
        cleanup(&space);
    }

    #[tokio::test]
    async fn get_shares_in_space_missing_dir_is_empty() {
        let space = temp_space();
        let listed = get_shares_in_space(&shares_root(&space), &space)
            .await
            .unwrap();
        assert!(listed.is_empty());
        cleanup(&space);
    }

    #[tokio::test]
    async fn get_shares_in_space_returns_all_saved_shares() {
        let space = temp_space();
        Share::new(
            "holden",
            &space.name,
            "photos.share.caby",
            "photos",
            vec![],
            vec![],
            None,
        )
        .save(&shares_root(&space), &space, &PathGuard::test().await)
        .await
        .unwrap();
        Share::new(
            "amos",
            &space.name,
            "docs.share.caby",
            "docs",
            vec![],
            vec![],
            None,
        )
        .save(&shares_root(&space), &space, &PathGuard::test().await)
        .await
        .unwrap();

        let listed = get_shares_in_space(&shares_root(&space), &space)
            .await
            .unwrap();
        assert_eq!(listed.len(), 2);

        cleanup(&space);
    }

    #[tokio::test]
    async fn delete_removes_and_is_idempotent() {
        let space = temp_space();
        let root = shares_root(&space);
        let share = sample(vec![], vec![]);
        share
            .save(&root, &space, &PathGuard::test().await)
            .await
            .unwrap();

        Share::delete(&root, &space, &share.id, &PathGuard::test().await)
            .await
            .unwrap();
        assert!(read_route(&root, &share.id).await.unwrap().is_none());

        // deleting an already-absent share is a no-op success
        Share::delete(&root, &space, &share.id, &PathGuard::test().await)
            .await
            .unwrap();

        cleanup(&space);
    }

    #[tokio::test]
    async fn delete_removes_the_live_spec_and_prunes_meta() {
        let space = temp_space();
        let share = sample(vec![], vec![]);
        let spec_path = Path::new(&share.spec_path);

        let spec_live = space.join(SpaceDir::LIVE, spec_path).unwrap();
        fs::create_dir_all(spec_live.parent().unwrap())
            .await
            .unwrap();
        fs::write(&spec_live, "guest_flows:\n  - permissions: [view]")
            .await
            .unwrap();
        let root = shares_root(&space);
        share
            .save(&root, &space, &PathGuard::test().await)
            .await
            .unwrap();

        let meta_dir = space.join(SpaceDir::META, spec_path).unwrap();
        assert!(fs::try_exists(&spec_live).await.unwrap());
        assert!(fs::try_exists(&meta_dir).await.unwrap());

        Share::delete(&root, &space, &share.id, &PathGuard::test().await)
            .await
            .unwrap();

        assert!(
            !fs::try_exists(&spec_live).await.unwrap(),
            "live spec left behind"
        );
        assert!(
            !fs::try_exists(&meta_dir).await.unwrap(),
            "empty meta dir left behind"
        );

        cleanup(&space);
    }
}
