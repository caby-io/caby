use anyhow::{anyhow, Context};
use chrono::{DateTime, Duration, Utc};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{auth::oidc::OIDC_DIR, Result};

const FLOW_STATE_TTL_MINUTES: i64 = 5;
const FLOW_STATE_FILE_PREFIX: &str = "flow_";
const STATE_MAX_LEN: usize = 200;

pub struct FlowState {
    pub pkce_verifier: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}

fn validate_state(state: &str) -> Result<()> {
    if state.is_empty() {
        return Err(anyhow!("OIDC state is empty"));
    }
    if state.len() > STATE_MAX_LEN {
        return Err(anyhow!("OIDC state exceeds maximum length"));
    }
    if !state
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(anyhow!(
            "OIDC state contains invalid characters, must be: A-Z a-z 0-9 _ -"
        ));
    }
    Ok(())
}

fn flow_state_path(home_path: &Path, state: &str) -> PathBuf {
    home_path
        .join(OIDC_DIR)
        .join(format!("{}{}", FLOW_STATE_FILE_PREFIX, state))
}

impl FlowState {
    pub async fn write(
        home_path: &Path,
        state: &str,
        pkce_verifier: &str,
        nonce: &str,
    ) -> Result<()> {
        validate_state(state)?;
        let expires_at = Utc::now() + Duration::minutes(FLOW_STATE_TTL_MINUTES);
        let content = format!("{}\n{}\n{}", pkce_verifier, nonce, expires_at.to_rfc3339());
        let path = flow_state_path(home_path, state);
        fs::write(&path, content)
            .await
            .with_context(|| format!("could not write OIDC flow state to {:?}", path))?;
        Ok(())
    }

    pub async fn take(home_path: &Path, state: &str) -> Result<Self> {
        validate_state(state)?;
        let path = flow_state_path(home_path, state);

        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("could not read OIDC flow state {:?}", path))?;

        fs::remove_file(&path)
            .await
            .with_context(|| format!("could not claim OIDC flow state {:?}", path))?;

        let mut lines = content.lines();

        let pkce_verifier = lines
            .next()
            .ok_or_else(|| anyhow!("OIDC flow state missing pkce_verifier line"))?
            .to_string();

        let nonce = lines
            .next()
            .ok_or_else(|| anyhow!("OIDC flow state missing nonce line"))?
            .to_string();

        let expires_at_str = lines
            .next()
            .ok_or_else(|| anyhow!("OIDC flow state missing expires_at line"))?;

        let expires_at = DateTime::parse_from_rfc3339(expires_at_str)
            .map_err(|err| anyhow!(err).context("could not parse OIDC flow state expires_at"))?
            .with_timezone(&Utc);

        if Utc::now() > expires_at {
            return Err(anyhow!("OIDC flow state expired"));
        }

        Ok(Self {
            pkce_verifier,
            nonce,
            expires_at,
        })
    }
}
