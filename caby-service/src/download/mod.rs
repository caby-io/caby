use std::{str::FromStr, time::Duration};

use crate::Result;
use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jiff::Timestamp;
use rand::RngExt;
use serde::Serialize;

const DOWNLOAD_LIFETIME: Duration = Duration::from_mins(5);

#[derive(Serialize)]
pub struct Token {
    pub value: String,
    pub space: String,
    pub file_paths: Vec<String>,
    pub issued_at: Timestamp,
    pub expires_at: Timestamp,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        Timestamp::now() > self.expires_at
    }

    pub fn to_file_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            self.value,
            self.space,
            self.file_paths.join("\0"),
            self.issued_at,
            self.expires_at,
        )
    }

    pub fn new(space: &str, file_paths: Vec<String>) -> Result<Self> {
        let mut bytes = [0u8; 32];
        rand::rng().fill(&mut bytes);

        let now = Timestamp::now();
        let expires_at = now
            .checked_add(DOWNLOAD_LIFETIME)
            .context("download token expiry is out of valid range")?;

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
        let issued_at: Timestamp = lines
            .next()
            .ok_or_else(|| anyhow!("could not read issued_at line from download file"))?
            .parse()
            .context("could not parse issued_at from download file")?;
        let expires_at: Timestamp = lines
            .next()
            .ok_or_else(|| anyhow!("could not read expires_at line from download file"))?
            .parse()
            .context("could not parse expires_at from download file")?;

        Ok(Self {
            value,
            space,
            file_paths,
            issued_at,
            expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHRONO_ERA_FILE: &str = "tok3n\nrocinante\na/one.jpg\0b/two.jpg\n2026-01-02T03:04:05.678901234+00:00\n2026-01-02T03:09:05.678901234+00:00";
    const JIFF_ERA_FILE: &str = "tok3n\nrocinante\na/one.jpg\0b/two.jpg\n2026-01-02T03:04:05.678901234Z\n2026-01-02T03:09:05.678901234Z";

    #[test]
    fn parses_offset_and_zulu_download_files_identically() {
        let chrono_era = Token::from_str(CHRONO_ERA_FILE).unwrap();
        let jiff_era = Token::from_str(JIFF_ERA_FILE).unwrap();

        assert_eq!(chrono_era.space, "rocinante");
        assert_eq!(chrono_era.file_paths, vec!["a/one.jpg", "b/two.jpg"]);
        assert_eq!(chrono_era.issued_at, jiff_era.issued_at);
        assert_eq!(chrono_era.expires_at, jiff_era.expires_at);
    }

    #[test]
    fn round_trips_through_file_string() {
        let token = Token::new(
            "rocinante",
            vec!["a/one.jpg".to_owned(), "b/two.jpg".to_owned()],
        )
        .unwrap();
        let parsed = Token::from_str(&token.to_file_string()).unwrap();

        assert_eq!(parsed.value, token.value);
        assert_eq!(parsed.space, token.space);
        assert_eq!(parsed.file_paths, token.file_paths);
        assert_eq!(parsed.issued_at, token.issued_at);
        assert_eq!(parsed.expires_at, token.expires_at);
    }
}
