use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use path_clean::PathClean;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::User,
    guest::Guest,
    space::{Space, SpaceDir},
    user::{try_hash_password, Account, Permission},
    Result,
};

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
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Share {
    pub id: String,
    pub owner_id: String,
    pub space: String,
    pub root_entry: String,

    pub account_allowlist: HashMap<String, Grant>,
    pub guest_allowlist: HashMap<String, Grant>,

    pub account_flows: Vec<ShareAccessFlow>,
    pub guest_flows: Vec<ShareAccessFlow>,

    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
struct ShareConfigFile {
    id: String,
    owner_id: String,
    space: String,
    root_entry: String,

    #[serde(default)]
    account_allowlist: BTreeMap<String, Grant>,
    #[serde(default)]
    guest_allowlist: BTreeMap<String, Grant>,

    #[serde(default)]
    account_flows: Vec<ShareAccessFlow>,
    #[serde(default)]
    guest_flows: Vec<ShareAccessFlow>,

    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl From<ShareConfigFile> for Share {
    fn from(stored: ShareConfigFile) -> Self {
        Share {
            id: stored.id,
            owner_id: stored.owner_id,
            space: stored.space,
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

impl From<&Share> for ShareConfigFile {
    fn from(share: &Share) -> Self {
        ShareConfigFile {
            id: share.id.clone(),
            owner_id: share.owner_id.clone(),
            space: share.space.clone(),
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

fn share_path(space: &Space, id: &str) -> Result<PathBuf> {
    space.join(SpaceDir::SHARES, Path::new(&format!("{id}.json")))
}

async fn load_share_file(path: &Path) -> Result<Share> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("could not read share file {:?}", path))?;
    let stored: ShareConfigFile = serde_json::from_str(&content)
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

        match load_share_file(&path).await {
            Ok(share) => shares.push(share),
            Err(err) => warn!("skipping unreadable share file {:?}: {:#}", path, err),
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
            Self::Password { hash } => {
                let parsed = argon2::PasswordHash::new(hash)
                    .map_err(|err| anyhow!("could not parse share password hash: {}", err))?;
                Ok(Argon2::default()
                    .verify_password(plaintext.as_bytes(), &parsed)
                    .is_ok())
            }
        }
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

impl Share {
    pub fn new(
        owner_id: &str,
        space: &str,
        root_entry: &str,
        account_flows: Vec<ShareAccessFlow>,
        guest_flows: Vec<ShareAccessFlow>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            owner_id: owner_id.to_owned(),
            space: space.to_owned(),
            root_entry: root_entry.to_owned(),
            account_allowlist: HashMap::new(),
            guest_allowlist: HashMap::new(),
            account_flows,
            guest_flows,
            created_at: Utc::now(),
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(at) => Utc::now() > at,
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
                created_at: Utc::now(),
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

    pub async fn save(&self, space: &Space) -> Result<()> {
        let path = share_path(space, &self.id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("could not create shares dir {:?}", parent))?;
        }

        let stored = ShareConfigFile::from(self);
        let serialized =
            serde_json::to_string_pretty(&stored).context("could not serialize share")?;
        fs::write(&path, serialized)
            .await
            .with_context(|| format!("could not write share file {:?}", path))?;

        Ok(())
    }

    pub async fn load(space: &Space, id: &str) -> Result<Option<Share>> {
        let path = share_path(space, id)?;
        if !fs::try_exists(&path)
            .await
            .with_context(|| format!("could not check share file {:?}", path))?
        {
            return Ok(None);
        }

        Ok(Some(load_share_file(&path).await?))
    }

    pub async fn delete(space: &Space, id: &str) -> Result<()> {
        let path = share_path(space, id)?;
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => {
                Err(anyhow!(err).context(format!("could not delete share file {:?}", path)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn sample(account_flows: Vec<ShareAccessFlow>, guest_flows: Vec<ShareAccessFlow>) -> Share {
        Share::new("suhaib", "home", "photos", account_flows, guest_flows, None)
    }

    fn temp_space() -> Space {
        Space {
            name: "home".to_owned(),
            display: "Home".to_owned(),
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
    fn is_expired_reflects_expiry() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.is_expired());
        share.expires_at = Some(Utc::now() - Duration::minutes(1));
        assert!(share.is_expired());
        share.expires_at = Some(Utc::now() + Duration::minutes(1));
        assert!(!share.is_expired());
    }

    #[tokio::test]
    async fn save_load_round_trip() {
        let space = temp_space();
        let share = Share::new(
            "suhaib",
            &space.name,
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

    #[tokio::test]
    async fn load_missing_is_none() {
        let space = temp_space();
        let loaded = Share::load(&space, "does-not-exist").await.unwrap();
        assert_eq!(loaded, None);
        cleanup(&space);
    }

    #[tokio::test]
    async fn list_in_space_missing_dir_is_empty() {
        let space = temp_space();
        let listed = get_shares_in_space(&space).await.unwrap();
        assert!(listed.is_empty());
        cleanup(&space);
    }

    #[tokio::test]
    async fn list_in_space_returns_all_saved_shares() {
        let space = temp_space();
        Share::new("suhaib", &space.name, "photos", vec![], vec![], None)
            .save(&space)
            .await
            .unwrap();
        Share::new("other", &space.name, "docs", vec![], vec![], None)
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
            created_at: Utc::now(),
        }
    }

    #[test]
    fn open_grants_requires_open_auth_and_permission() {
        assert!(open_flow(&[Permission::View]).open_grants(Permission::View));
        assert!(!open_flow(&[Permission::View]).open_grants(Permission::Download));
        assert!(!password_flow(&[Permission::View]).open_grants(Permission::View));
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
        assert!(share.can_account(&account("member"), Permission::Write));
        assert!(!share.can_account(&account("member"), Permission::Delete));
    }

    #[test]
    fn can_account_falls_through_to_open_guest_access() {
        let share = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(share.can_account(&account("member"), Permission::View));
    }

    #[test]
    fn can_account_via_allowlist() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.can_account(&account("member"), Permission::Delete));

        share
            .account_allowlist
            .insert("member".to_owned(), grant(&[Permission::Delete]));
        assert!(share.can_account(&account("member"), Permission::Delete));
        assert!(!share.can_account(&account("other"), Permission::Delete));
    }

    #[test]
    fn owner_bypasses_flows_and_allowlist() {
        let share = sample(vec![], vec![]);
        assert!(share.can_account(&account("suhaib"), Permission::Delete));
        assert!(!share.can_account(&account("other"), Permission::Delete));
    }

    #[test]
    fn can_admin_only_for_owner() {
        let share = sample(vec![], vec![]);
        assert!(share.can_admin(&account("suhaib")));
        assert!(!share.can_admin(&account("other")));
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
            &User::Account(account("suhaib")),
            BTreeSet::from([Permission::Delete]),
        );
        assert!(share.account_allowlist.contains_key("suhaib"));
    }
}
