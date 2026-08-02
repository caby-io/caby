use std::{str::FromStr, time::Duration};

use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jiff::Timestamp;
use rand::RngExt;
use serde::Serialize;

use crate::{guest::Guest, user::Account, Result};

pub mod oidc;

const SESSION_LIFETIME: Duration = Duration::from_hours(24);

#[derive(Serialize)]
pub struct Token {
    pub value: String,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
}

impl FromStr for Token {
    type Err = crate::Error;

    fn from_str(content: &str) -> Result<Self> {
        let mut lines = content.lines();

        let value = lines
            .next()
            .ok_or_else(|| anyhow!("could not read token value line from session file"))?
            .to_string();
        let issued_at: Timestamp = lines
            .next()
            .ok_or_else(|| anyhow!("could not read issued_at line from session file"))?
            .parse()
            .context("could not parse issued_at from session file")?;
        let expires_at: Timestamp = lines
            .next()
            .ok_or_else(|| anyhow!("could not read expires_at line from session file"))?
            .parse()
            .context("could not parse expires_at from session file")?;

        Ok(Self {
            value,
            issued_at,
            expires_at,
        })
    }
}

impl Token {
    pub fn is_expired(&self) -> bool {
        Timestamp::now() > self.expires_at
    }

    pub fn to_file_string(&self) -> String {
        format!("{}\n{}\n{}", self.value, self.issued_at, self.expires_at)
    }

    pub fn new() -> Result<Self> {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);

        let now = Timestamp::now();
        let expires_at = now
            .checked_add(SESSION_LIFETIME)
            .context("session token expiry is out of valid range")?;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHRONO_ERA_FILE: &str =
        "tok3n\n2026-01-02T03:04:05.678901234+00:00\n2026-01-03T03:04:05.678901234+00:00";
    const JIFF_ERA_FILE: &str =
        "tok3n\n2026-01-02T03:04:05.678901234Z\n2026-01-03T03:04:05.678901234Z";

    #[test]
    fn parses_offset_and_zulu_session_files_identically() {
        let chrono_era = Token::from_str(CHRONO_ERA_FILE).unwrap();
        let jiff_era = Token::from_str(JIFF_ERA_FILE).unwrap();

        assert_eq!(chrono_era.value, "tok3n");
        assert_eq!(chrono_era.issued_at, jiff_era.issued_at);
        assert_eq!(chrono_era.expires_at, jiff_era.expires_at);
    }

    #[test]
    fn round_trips_through_file_string() {
        let token = Token::new().unwrap();
        let parsed = Token::from_str(&token.to_file_string()).unwrap();

        assert_eq!(parsed.value, token.value);
        assert_eq!(parsed.issued_at, token.issued_at);
        assert_eq!(parsed.expires_at, token.expires_at);
    }

    #[test]
    fn serializes_expiry_as_zulu_rfc3339() {
        let token = Token::from_str(CHRONO_ERA_FILE).unwrap();
        let json = serde_json::to_string(&token).unwrap();

        assert!(json.contains(r#""expires_at":"2026-01-03T03:04:05.678901234Z""#));
    }
}
