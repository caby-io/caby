use std::{
    collections::BTreeSet,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use argon2::{Argon2, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    space::{Space, SpaceDir},
    user::try_hash_password,
    Result,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShareAuth {
    Open,
    Password { hash: String },
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SharePermission {
    View,
    Download,
    Upload,
    CreateDir,
    Delete,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareLimits {
    pub max_file_bytes: Option<u64>,
    pub max_bytes_per_day: Option<u64>,
    pub max_files_per_day: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareAccess {
    pub auth: ShareAuth,
    pub permissions: BTreeSet<SharePermission>,
    pub limits: Option<ShareLimits>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Share {
    pub id: String,
    pub owner: String,
    pub space: String,
    pub entries_root: String,
    pub member_access: Option<ShareAccess>,
    pub public_access: Option<ShareAccess>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn share_path(space: &Space, id: &str) -> Result<PathBuf> {
    space.join(SpaceDir::SHARES, Path::new(&format!("{id}.json")))
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

impl Share {
    pub fn new(
        owner: &str,
        space: &str,
        entries_root: &str,
        member_access: Option<ShareAccess>,
        public_access: Option<ShareAccess>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            owner: owner.to_owned(),
            space: space.to_owned(),
            entries_root: entries_root.to_owned(),
            member_access,
            public_access,
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

    pub async fn save(&self, space: &Space) -> Result<()> {
        let path = share_path(space, &self.id)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("could not create shares dir {:?}", parent))?;
        }

        let serialized = serde_json::to_string_pretty(self).context("could not serialize share")?;
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

        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("could not read share file {:?}", path))?;
        let share: Share = serde_json::from_str(&content)
            .with_context(|| format!("could not parse share file {:?}", path))?;

        Ok(Some(share))
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

    fn sample(member_access: Option<ShareAccess>, public_access: Option<ShareAccess>) -> Share {
        Share::new(
            "suhaib",
            "home",
            "photos",
            member_access,
            public_access,
            None,
        )
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
        let a = sample(None, None);
        let b = sample(None, None);
        assert!(!a.id.is_empty());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn is_expired_reflects_expiry() {
        let mut share = sample(None, None);
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
            None,
            Some(ShareAccess {
                auth: ShareAuth::Open,
                permissions: BTreeSet::from([SharePermission::View, SharePermission::Download]),
                limits: Some(ShareLimits {
                    max_file_bytes: Some(1024),
                    max_bytes_per_day: None,
                    max_files_per_day: Some(10),
                }),
            }),
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
    async fn delete_removes_and_is_idempotent() {
        let space = temp_space();
        let share = sample(None, None);
        share.save(&space).await.unwrap();

        Share::delete(&space, &share.id).await.unwrap();
        assert_eq!(Share::load(&space, &share.id).await.unwrap(), None);

        // deleting an already-absent share is a no-op success
        Share::delete(&space, &share.id).await.unwrap();

        cleanup(&space);
    }
}
