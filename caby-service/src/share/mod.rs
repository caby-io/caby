use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jiff::Timestamp;
use path_clean::PathClean;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::User,
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    guest::Guest,
    space::{Space, SpaceDir},
    user::{try_hash_password, Account, Permission},
    Result,
};

pub mod spec;

pub use spec::{spec_root, ShareSpec, SpecAuth, SpecFlow};

pub const CABY_SHARE_STATE_FILE: &str = "share.json";
const SHARE_DEFAULT_FILTER: &[fn(&str) -> bool] = &[is_share_spec];

fn is_share_spec(name: &str) -> bool {
    has_ext(Path::new(name), CABY_SHARE_SPEC_EXT)
}

// todo: support files + dirs
pub fn is_filtered(name: &str) -> bool {
    SHARE_DEFAULT_FILTER.iter().any(|rule| rule(name))
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShareAuth {
    Open,
    Password { hash: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareLimits {
    pub max_file_bytes: Option<u64>,
    pub max_bytes_per_day: Option<u64>,
    pub max_files_per_day: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareAccessFlow {
    pub auth: ShareAuth,
    pub permissions: BTreeSet<Permission>,
    pub limits: Option<ShareLimits>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Grant {
    pub permissions: BTreeSet<Permission>,
    pub created_at: Timestamp,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Share {
    pub id: String,
    pub owner_id: String,
    pub space: String,
    pub spec_path: String,
    pub root_entry: String,

    pub account_allowlist: HashMap<String, Grant>,
    pub guest_allowlist: HashMap<String, Grant>,

    pub account_flows: Vec<ShareAccessFlow>,
    pub guest_flows: Vec<ShareAccessFlow>,

    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

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

fn route_path(space: &Space, id: &str) -> Result<PathBuf> {
    space.join(SpaceDir::SHARES, Path::new(&format!("{id}.json")))
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

pub async fn get_shares_in_space(space: &Space) -> Result<Vec<Share>> {
    let dir = space.shares();
    let mut read_dir = match fs::read_dir(&dir).await {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read shares dir {:?}", dir)))
        }
    };

    let mut shares = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .with_context(|| format!("could not read shares dir {:?}", dir))?
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };

        match Share::load(space, id).await {
            Ok(Some(share)) => shares.push(share),
            Ok(None) => warn!("skipping dangling share route {:?}", path),
            Err(err) => warn!("skipping unreadable share route {:?}: {:#}", path, err),
        }
    }

    Ok(shares)
}

impl ShareAuth {
    pub fn password(plaintext: &str) -> Result<Self> {
        Ok(Self::Password {
            hash: try_hash_password(plaintext)?,
        })
    }

    pub fn try_verify(&self, plaintext: &str) -> Result<bool> {
        match self {
            Self::Open => Ok(true),
            Self::Password { hash } => verify_hash(hash, plaintext),
        }
    }
}

pub(crate) enum HashFormat {
    Argon2,
    Bcrypt,
    Sha512Crypt,
}

pub(crate) fn hash_format(hash: &str) -> Option<HashFormat> {
    if hash.starts_with("$argon2") {
        Some(HashFormat::Argon2)
    } else if hash.starts_with("$2a$") || hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        Some(HashFormat::Bcrypt)
    } else if hash.starts_with("$6$") {
        Some(HashFormat::Sha512Crypt)
    } else {
        None
    }
}

fn verify_hash(hash: &str, plaintext: &str) -> Result<bool> {
    match hash_format(hash) {
        Some(HashFormat::Argon2) => {
            let parsed = argon2::PasswordHash::new(hash)
                .map_err(|err| anyhow!("could not parse argon2 hash: {}", err))?;
            Ok(Argon2::default()
                .verify_password(plaintext.as_bytes(), &parsed)
                .is_ok())
        }
        Some(HashFormat::Bcrypt) => bcrypt::verify(plaintext, hash)
            .map_err(|err| anyhow!("could not verify bcrypt hash: {}", err)),
        Some(HashFormat::Sha512Crypt) => Ok(sha_crypt::sha512_check(plaintext, hash).is_ok()),
        None => Err(anyhow!("unsupported password hash format")),
    }
}

impl ShareAccessFlow {
    pub fn grants(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn open_grants(&self, permission: Permission) -> bool {
        matches!(self.auth, ShareAuth::Open) && self.grants(permission)
    }
}

impl TryFrom<SpecFlow> for ShareAccessFlow {
    type Error = crate::Error;

    fn try_from(flow: SpecFlow) -> Result<Self> {
        let auth = match flow.auth {
            SpecAuth::Open => ShareAuth::Open,
            SpecAuth::Password(plaintext) => ShareAuth::password(&plaintext)?,
            SpecAuth::Hash(hash) => {
                if hash_format(&hash).is_none() {
                    return Err(anyhow!(
                        "unsupported password hash format (use argon2 '$argon2id$', bcrypt '$2b$', or sha512-crypt '$6$')"
                    ));
                }
                ShareAuth::Password { hash }
            }
        };
        Ok(ShareAccessFlow {
            auth,
            permissions: flow.permissions,
            limits: flow.limits,
        })
    }
}

impl Share {
    pub fn new(
        owner_id: &str,
        space: &str,
        spec_path: &str,
        root_entry: &str,
        account_flows: Vec<ShareAccessFlow>,
        guest_flows: Vec<ShareAccessFlow>,
        expires_at: Option<Timestamp>,
    ) -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            owner_id: owner_id.to_owned(),
            space: space.to_owned(),
            spec_path: spec_path.to_owned(),
            root_entry: root_entry.to_owned(),
            account_allowlist: HashMap::new(),
            guest_allowlist: HashMap::new(),
            account_flows,
            guest_flows,
            created_at: Timestamp::now(),
            expires_at,
        }
    }

    pub fn from_spec(
        space: &str,
        spec_path: &Path,
        spec: ShareSpec,
        existing: Option<Share>,
    ) -> Result<Self> {
        let root = spec_root(spec_path)
            .ok_or_else(|| anyhow!("not a share spec path: {:?}", spec_path))?;
        let root_entry = root.to_string_lossy().into_owned();
        let spec_path = spec_path.to_string_lossy().into_owned();

        let account_flows = spec
            .account_flows
            .into_iter()
            .map(ShareAccessFlow::try_from)
            .collect::<Result<Vec<_>>>()?;
        let guest_flows = spec
            .guest_flows
            .into_iter()
            .map(ShareAccessFlow::try_from)
            .collect::<Result<Vec<_>>>()?;

        let share = match existing {
            Some(prev) => Share {
                id: prev.id,
                owner_id: prev.owner_id,
                space: space.to_owned(),
                spec_path,
                root_entry,
                account_allowlist: prev.account_allowlist,
                guest_allowlist: prev.guest_allowlist,
                account_flows,
                guest_flows,
                created_at: prev.created_at,
                expires_at: spec.expires_at,
            },
            None => Share::new(
                "",
                space,
                &spec_path,
                &root_entry,
                account_flows,
                guest_flows,
                spec.expires_at,
            ),
        };

        Ok(share)
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(at) => Timestamp::now() > at,
            None => false,
        }
    }

    pub fn can_account(&self, account: &Account, permission: Permission) -> bool {
        if self.can_admin(account) {
            return true;
        }

        if let Some(grant) = self.account_allowlist.get(&account.name) {
            if grant.permissions.contains(&permission) {
                return true;
            }
        }

        // check if there's open access for accounts
        self.account_flows
            .iter()
            .any(|flow| flow.open_grants(permission))
            || self.can_any_guest(permission)
    }

    pub fn can_any_guest(&self, permission: Permission) -> bool {
        self.guest_flows
            .iter()
            .any(|flow| flow.open_grants(permission))
    }

    pub fn can_guest(&self, guest: &Guest, permission: Permission) -> bool {
        if let Some(grant) = self.guest_allowlist.get(&guest.id) {
            if grant.permissions.contains(&permission) {
                return true;
            }
        }

        self.can_any_guest(permission)
    }

    pub fn can_admin(&self, account: &Account) -> bool {
        account.name == self.owner_id
    }

    pub fn grant(&mut self, user: &User, permissions: BTreeSet<Permission>) {
        let (allowlist, id) = match user {
            User::Account(account) => (&mut self.account_allowlist, account.name.clone()),
            User::Guest(guest) => (&mut self.guest_allowlist, guest.id.clone()),
        };
        allowlist.insert(
            id,
            Grant {
                permissions,
                created_at: Timestamp::now(),
            },
        );
    }

    pub fn scope_path(&self, space: &Space, rel: &Path) -> Result<PathBuf> {
        let root = PathBuf::from(&self.root_entry).clean();
        let scoped = root.join(rel.clean()).clean();

        let live = space.join(SpaceDir::LIVE, &scoped)?;
        let root_live = space.live().join(&root);
        if !live.starts_with(&root_live) {
            return Err(anyhow!("path escapes share root"));
        }

        Ok(scoped)
    }

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

    pub async fn save(&self, space: &Space) -> Result<()> {
        let spec_path = Path::new(&self.spec_path);
        self.write_to(&state_path(space, spec_path)?).await?;
        write_route(space, &self.id, &self.spec_path).await
    }

    pub async fn load(space: &Space, id: &str) -> Result<Option<Share>> {
        let Some(spec_path) = read_route(space, id).await? else {
            return Ok(None);
        };
        load_state(space, Path::new(&spec_path)).await
    }

    pub async fn delete(space: &Space, id: &str) -> Result<()> {
        let Some(spec_path) = read_route(space, id).await? else {
            return Ok(());
        };
        let spec_path = Path::new(&spec_path);

        remove_spec(space, spec_path).await?;
        remove_state(space, spec_path).await?;
        remove_route(space, id).await
    }
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
    spec_path: String,
}

async fn read_route(space: &Space, id: &str) -> Result<Option<String>> {
    let path = route_path(space, id)?;
    match fs::read_to_string(&path).await {
        Ok(content) => {
            let route: RouteFile = serde_json::from_str(&content)
                .with_context(|| format!("could not parse share route {:?}", path))?;
            Ok(Some(route.spec_path))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(anyhow!(err).context(format!("could not read share route {:?}", path))),
    }
}

async fn write_route(space: &Space, id: &str, spec_path: &str) -> Result<()> {
    let path = route_path(space, id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("could not create shares dir {:?}", parent))?;
    }

    let serialized = serde_json::to_string_pretty(&RouteFile {
        spec_path: spec_path.to_owned(),
    })
    .context("could not serialize share route")?;
    fs::write(&path, serialized)
        .await
        .with_context(|| format!("could not write share route {:?}", path))?;

    Ok(())
}

async fn remove_route(space: &Space, id: &str) -> Result<()> {
    let path = route_path(space, id)?;
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow!(err).context(format!("could not remove share route {:?}", path))),
    }
}

pub async fn remove_routes_for_spec(space: &Space, spec_path: &Path) -> Result<()> {
    let dir = space.shares();
    let mut read_dir = match fs::read_dir(&dir).await {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(anyhow!(err).context(format!("could not read shares dir {:?}", dir)))
        }
    };

    let target = spec_path.to_string_lossy();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .with_context(|| format!("could not read shares dir {:?}", dir))?
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        match read_route(space, id).await {
            Ok(Some(spec)) if spec == target => remove_route(space, id).await?,
            Ok(_) => {}
            Err(err) => warn!("skipping unreadable share route {:?}: {:#}", path, err),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

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

    fn cleanup(space: &Space) {
        let _ = std::fs::remove_dir_all(&space.path);
    }

    #[test]
    fn new_generates_unique_non_empty_ids() {
        let a = sample(vec![], vec![]);
        let b = sample(vec![], vec![]);
        assert!(!a.id.is_empty());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn hides_share_specs_from_shares() {
        assert!(is_filtered("photos.share.caby"));
        assert!(is_filtered("a.b.share.caby"));
        assert!(!is_filtered("photo.jpg"));
        assert!(!is_filtered("notes.txt"));
        assert!(!is_filtered("share.caby"));
    }

    #[test]
    fn is_expired_reflects_expiry() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.is_expired());
        share.expires_at = Some(Timestamp::now() - SignedDuration::from_mins(1));
        assert!(share.is_expired());
        share.expires_at = Some(Timestamp::now() + SignedDuration::from_mins(1));
        assert!(!share.is_expired());
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

        share.save(&space).await.unwrap();
        let loaded = Share::load(&space, &share.id).await.unwrap();
        assert_eq!(loaded, Some(share));

        cleanup(&space);
    }

    fn spec(guest_perms: &[Permission]) -> ShareSpec {
        ShareSpec {
            account_flows: vec![],
            guest_flows: vec![SpecFlow {
                auth: SpecAuth::Open,
                permissions: guest_perms.iter().copied().collect(),
                limits: None,
            }],
            expires_at: None,
        }
    }

    #[test]
    fn from_spec_mints_an_id_then_carries_it_across_edits() {
        let spec_path = std::path::Path::new("photos/public.share.caby");
        let first =
            Share::from_spec("rocinante", spec_path, spec(&[Permission::View]), None).unwrap();
        assert!(!first.id.is_empty());
        assert_eq!(first.root_entry, "photos");
        assert_eq!(first.spec_path, "photos/public.share.caby");
        assert!(first.can_any_guest(Permission::View));

        let second = Share::from_spec(
            "rocinante",
            spec_path,
            spec(&[Permission::View, Permission::Download]),
            Some(first.clone()),
        )
        .unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(second.created_at, first.created_at);
        assert!(second.can_any_guest(Permission::Download));
    }

    #[tokio::test]
    async fn share_route_round_trip() {
        let space = temp_space();

        assert!(read_route(&space, "abc123").await.unwrap().is_none());

        write_route(&space, "abc123", "photos/trip.share.caby")
            .await
            .unwrap();
        assert_eq!(
            read_route(&space, "abc123").await.unwrap().as_deref(),
            Some("photos/trip.share.caby")
        );

        remove_route(&space, "abc123").await.unwrap();
        assert!(read_route(&space, "abc123").await.unwrap().is_none());

        cleanup(&space);
    }

    #[tokio::test]
    async fn load_missing_is_none() {
        let space = temp_space();
        let loaded = Share::load(&space, "does-not-exist").await.unwrap();
        assert_eq!(loaded, None);
        cleanup(&space);
    }

    #[tokio::test]
    async fn get_shares_in_space_missing_dir_is_empty() {
        let space = temp_space();
        let listed = get_shares_in_space(&space).await.unwrap();
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
        .save(&space)
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
        .save(&space)
        .await
        .unwrap();

        let listed = get_shares_in_space(&space).await.unwrap();
        assert_eq!(listed.len(), 2);

        cleanup(&space);
    }

    #[tokio::test]
    async fn delete_removes_and_is_idempotent() {
        let space = temp_space();
        let share = sample(vec![], vec![]);
        share.save(&space).await.unwrap();

        Share::delete(&space, &share.id).await.unwrap();
        assert_eq!(Share::load(&space, &share.id).await.unwrap(), None);

        // deleting an already-absent share is a no-op success
        Share::delete(&space, &share.id).await.unwrap();

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
        share.save(&space).await.unwrap();

        let meta_dir = space.join(SpaceDir::META, spec_path).unwrap();
        assert!(fs::try_exists(&spec_live).await.unwrap());
        assert!(fs::try_exists(&meta_dir).await.unwrap());

        Share::delete(&space, &share.id).await.unwrap();

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

    fn open_flow(perms: &[Permission]) -> ShareAccessFlow {
        ShareAccessFlow {
            auth: ShareAuth::Open,
            permissions: perms.iter().copied().collect(),
            limits: None,
        }
    }

    fn password_flow(perms: &[Permission]) -> ShareAccessFlow {
        ShareAccessFlow {
            auth: ShareAuth::Password {
                hash: "hash".to_owned(),
            },
            permissions: perms.iter().copied().collect(),
            limits: None,
        }
    }

    fn account(name: &str) -> Account {
        Account {
            name: name.to_owned(),
            path: PathBuf::from("/tmp"),
            email: None,
            activation_token: None,
            space_access: vec![],
        }
    }

    fn grant(perms: &[Permission]) -> Grant {
        Grant {
            permissions: perms.iter().copied().collect(),
            created_at: Timestamp::now(),
        }
    }

    #[test]
    fn open_grants_requires_open_auth_and_permission() {
        assert!(open_flow(&[Permission::View]).open_grants(Permission::View));
        assert!(!open_flow(&[Permission::View]).open_grants(Permission::Download));
        assert!(!password_flow(&[Permission::View]).open_grants(Permission::View));
    }

    #[test]
    fn verify_accepts_argon2_bcrypt_and_sha512crypt() {
        // each is a hash of "sharepass" in its respective format
        for hash in [
            "$argon2id$v=19$m=19456,t=2,p=1$2wZGyG/UHzKi38a9Pydu2A$yPXx0OjYTOCjoKmSCfEKGBuoNkV6dClbHyYzyNzSGbY",
            "$2b$10$uy7PcJEUHqLpzk9H.QE0JeiCx0cs5MLXlyOdMkLnOwns1xqJPMcUK",
            "$6$abcd1234$thjHjXmLs1exuU707Xf57m1TIjqLDWtvxoVWS2L1S.m59iBJZlLujLGZxg24M/u3qEy2PDnqsYmT8l8PlziHq1",
        ] {
            let auth = ShareAuth::Password {
                hash: hash.to_owned(),
            };
            assert!(auth.try_verify("sharepass").unwrap(), "should accept: {hash}");
            assert!(!auth.try_verify("wrong").unwrap(), "should reject wrong pw: {hash}");
        }
    }

    #[test]
    fn verify_rejects_unsupported_hash_format() {
        assert!(hash_format("$argon2id$v=19$m=1$x$y").is_some());
        assert!(hash_format("$2b$10$abc").is_some());
        assert!(hash_format("$6$salt$hash").is_some());
        assert!(hash_format("plaintext").is_none());
        // bare sha256 hex is easy to make but unsalted/fast — not accepted
        let sha256hex = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
        assert!(hash_format(sha256hex).is_none());
        let auth = ShareAuth::Password {
            hash: sha256hex.to_owned(),
        };
        assert!(auth.try_verify("sharepass").is_err());
    }

    #[test]
    fn can_any_guest_only_from_open_guest_flows() {
        let open = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(open.can_any_guest(Permission::View));
        assert!(!open.can_any_guest(Permission::Download));

        let locked = sample(vec![], vec![password_flow(&[Permission::View])]);
        assert!(!locked.can_any_guest(Permission::View));
    }

    #[test]
    fn can_guest_via_open_flow() {
        let share = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(share.can_guest(&Guest::new(), Permission::View));
        assert!(!share.can_guest(&Guest::new(), Permission::Download));
    }

    #[test]
    fn can_guest_via_allowlist() {
        let guest = Guest::new();
        let mut share = sample(vec![], vec![]);
        assert!(!share.can_guest(&guest, Permission::Download));

        share
            .guest_allowlist
            .insert(guest.id.clone(), grant(&[Permission::Download]));
        assert!(share.can_guest(&guest, Permission::Download));
        assert!(!share.can_guest(&guest, Permission::Delete));
        assert!(!share.can_guest(&Guest::new(), Permission::Download));
    }

    #[test]
    fn can_account_via_open_account_flow() {
        let share = sample(vec![open_flow(&[Permission::Write])], vec![]);
        assert!(share.can_account(&account("naomi"), Permission::Write));
        assert!(!share.can_account(&account("naomi"), Permission::Delete));
    }

    #[test]
    fn can_account_falls_through_to_open_guest_access() {
        let share = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(share.can_account(&account("naomi"), Permission::View));
    }

    #[test]
    fn can_account_via_allowlist() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.can_account(&account("naomi"), Permission::Delete));

        share
            .account_allowlist
            .insert("naomi".to_owned(), grant(&[Permission::Delete]));
        assert!(share.can_account(&account("naomi"), Permission::Delete));
        assert!(!share.can_account(&account("marco"), Permission::Delete));
    }

    #[test]
    fn owner_bypasses_flows_and_allowlist() {
        let share = sample(vec![], vec![]);
        assert!(share.can_account(&account("holden"), Permission::Delete));
        assert!(!share.can_account(&account("marco"), Permission::Delete));
    }

    #[test]
    fn can_admin_only_for_owner() {
        let share = sample(vec![], vec![]);
        assert!(share.can_admin(&account("holden")));
        assert!(!share.can_admin(&account("marco")));
    }

    #[test]
    fn grant_routes_by_principal_kind() {
        let mut share = sample(vec![], vec![]);

        let guest = Guest::new();
        let guest_id = guest.id.clone();
        share.grant(&User::Guest(guest), BTreeSet::from([Permission::View]));
        assert!(share.guest_allowlist.contains_key(&guest_id));
        assert!(share.account_allowlist.is_empty());

        share.grant(
            &User::Account(account("holden")),
            BTreeSet::from([Permission::Delete]),
        );
        assert!(share.account_allowlist.contains_key("holden"));
    }
}
