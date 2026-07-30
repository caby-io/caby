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

    pub fn is_account(&self) -> bool {
        matches!(&self.user, User::Account(_))
    }

    pub fn is_guest(&self) -> bool {
        matches!(&self.user, User::Guest(_))
    }

    pub fn can_on_share(&self, share: &Share, permission: Permission) -> bool {
        match &self.user {
            User::Account(account) => share.can_account(account, permission),
            User::Guest(guest) => share.can_guest(guest, permission),
        }
    }
}

pub fn authorize_share(auth: Option<&AuthUser>, share: &Share, permission: Permission) -> bool {
    match auth {
        Some(user) => user.can_on_share(share, permission),
        None => share.can_any_guest(permission),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::PathBuf};

    use super::*;
    use crate::share::{ShareAccessFlow, ShareAuth};

    fn open_flow(permission: Permission) -> ShareAccessFlow {
        ShareAccessFlow {
            auth: ShareAuth::Open,
            permissions: BTreeSet::from([permission]),
            limits: None,
        }
    }

    fn open_account_share() -> Share {
        Share::new(
            "owner",
            "home",
            "photos",
            vec![open_flow(Permission::Write)],
            vec![],
            None,
        )
    }

    fn open_guest_share() -> Share {
        Share::new(
            "owner",
            "home",
            "photos",
            vec![],
            vec![open_flow(Permission::View)],
            None,
        )
    }

    fn account_auth(name: &str) -> AuthUser {
        AuthUser {
            token: "token".to_owned(),
            user: User::Account(Account {
                name: name.to_owned(),
                path: PathBuf::from("/tmp"),
                email: None,
                activation_token: None,
                space_access: vec![],
            }),
        }
    }

    fn guest_auth() -> AuthUser {
        AuthUser {
            token: "token".to_owned(),
            user: User::Guest(Guest::new()),
        }
    }

    #[test]
    fn anonymous_gets_open_guest_access() {
        let share = open_guest_share();
        assert!(authorize_share(None, &share, Permission::View));
        assert!(!authorize_share(None, &share, Permission::Download));
    }

    #[test]
    fn guest_principal_routes_to_can_guest() {
        let auth = guest_auth();
        assert!(auth.can_on_share(&open_guest_share(), Permission::View));
        assert!(!auth.can_on_share(&open_guest_share(), Permission::Download));
    }

    #[test]
    fn account_principal_routes_to_can_account() {
        let auth = account_auth("suhaib");
        assert!(auth.can_on_share(&open_account_share(), Permission::Write));
        assert!(!auth.can_on_share(&open_account_share(), Permission::Delete));
    }
}
