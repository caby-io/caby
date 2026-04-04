use std::str::FromStr;

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::RngExt;
use serde::Serialize;

use crate::Result;

pub mod auth_middleware;

#[derive(Serialize)]
pub struct Token {
    pub value: String,
    pub created_at: DateTime<Utc>,
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
        let created_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read created_at line from session file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse created_at from session file"))?
        .with_timezone(&Utc);
        let expires_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read expires_at line from session file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse expires_at from session file"))?
        .with_timezone(&Utc);

        return Ok(Self {
            value,
            created_at,
            expires_at,
        });
    }
}

impl Token {
    pub fn new() -> Result<Self> {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);

        let now = Utc::now();
        let expires_at = now + Duration::hours(24);

        Ok(Self {
            value: URL_SAFE_NO_PAD.encode(bytes),
            created_at: now,
            expires_at,
        })
    }
}
