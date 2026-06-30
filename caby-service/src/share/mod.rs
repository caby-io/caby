use std::collections::BTreeSet;

use anyhow::anyhow;
use argon2::{Argon2, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{user::try_hash_password, Result};

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShareAccess {
    pub auth: ShareAuth,
    pub permissions: BTreeSet<SharePermission>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
}
