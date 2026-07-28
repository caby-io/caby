use std::str::FromStr;

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::RngExt;
use serde::Serialize;

use crate::{
    guest::Guest,
    share::Share,
    user::{Account, Permission},
    Result,
};

pub mod oidc;

#[derive(Serialize)]
pub struct Token {
    pub value: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl FromStr for Token {
    type Err = crate::Error;

    fn from_str(content: &str) -> Result<Self> {
        let mut lines = content.lines();

        let value = lines
            .next()
            .ok_or_else(|| anyhow!("could not read token value line from session file"))?
            .to_string();
        let issued_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read issued_at line from session file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse issued_at from session file"))?
        .with_timezone(&Utc);
        let expires_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read expires_at line from session file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse expires_at from session file"))?
        .with_timezone(&Utc);

        Ok(Self {
            value,
            issued_at,
            expires_at,
        })
    }
}

impl Token {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn new() -> Result<Self> {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);

        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        Ok(Self {
            value: URL_SAFE_NO_PAD.encode(bytes),
            issued_at: now,
            expires_at,
        })
    }
}

pub enum User {
    Account(Account),
    Guest(Guest),
}

pub struct AuthUser {
    pub token: String,
    pub user: User,
}

impl AuthUser {
    pub fn as_account(&self) -> Option<&Account> {
        match &self.user {
            User::Account(account) => Some(account),
            User::Guest(_) => None,
        }
    }

    pub fn is_authorized_share(&self, share: &Share, permission: Permission) -> bool {
        match &self.user {
            User::Account(account) => {
                let is_space_member = account.space_access.iter().any(|s| s.name == share.space);
                if is_space_member {
                    share
                        .member_flow
                        .as_ref()
                        .is_some_and(|flow| flow.grants(permission))
                } else if share.is_member(&account.name) {
                    share
                        .guest_flow
                        .as_ref()
                        .is_some_and(|flow| flow.grants(permission))
                } else {
                    guest_authorized(share, permission, None)
                }
            }
            User::Guest(guest) => {
                let fingerprint = guest
                    .access_for(&share.space, &share.id)
                    .and_then(|access| access.password_fingerprint);
                guest_authorized(share, permission, fingerprint)
            }
        }
    }
}

fn guest_authorized(share: &Share, permission: Permission, fingerprint: Option<u64>) -> bool {
    share
        .guest_flow
        .as_ref()
        .is_some_and(|flow| flow.grants(permission) && flow.auth.satisfied_by(fingerprint))
}

pub fn authorize_share(auth: Option<&AuthUser>, share: &Share, permission: Permission) -> bool {
    match auth {
        Some(user) => user.is_authorized_share(share, permission),
        None => guest_authorized(share, permission, None),
    }
}
