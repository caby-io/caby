use std::str::FromStr;

use crate::Result;
use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::RngExt;
use serde::Serialize;

#[derive(Serialize)]
pub struct Token {
    pub value: String,
    pub space: String,
    pub file_paths: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn to_file_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.value,
            self.space,
            self.file_paths.join("\0"),
            self.issued_at.to_rfc3339(),
            self.expires_at.to_rfc3339(),
        )
    }

    pub fn new(space: &str, file_paths: Vec<String>) -> Result<Self> {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);

        let now = Utc::now();
        let expires_at = now + Duration::minutes(5);

        Ok(Self {
            value: URL_SAFE_NO_PAD.encode(bytes),
            space: space.to_string(),
            file_paths,
            issued_at: now,
            expires_at,
        })
    }
}

impl FromStr for Token {
    type Err = crate::Error;

    fn from_str(content: &str) -> Result<Self> {
        let mut lines = content.lines();

        let value = lines
            .next()
            .ok_or_else(|| anyhow!("could not read token value line from download file"))?
            .to_string();
        let space = lines
            .next()
            .ok_or_else(|| anyhow!("could not read space line from download file"))?
            .to_string();
        let file_paths = lines
            .next()
            .ok_or_else(|| anyhow!("could not read file paths line from download file"))?
            .split('\0')
            .map(String::from)
            .collect();
        let issued_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read issued_at line from download file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse issued_at from download file"))?
        .with_timezone(&Utc);
        let expires_at = DateTime::parse_from_rfc3339(
            lines
                .next()
                .ok_or_else(|| anyhow!("could not read expires_at line from download file"))?,
        )
        .map_err(|err| anyhow!(err).context("could not parse expires_at from download file"))?
        .with_timezone(&Utc);

        return Ok(Self {
            value,
            space,
            file_paths,
            issued_at,
            expires_at,
        });
    }
}
