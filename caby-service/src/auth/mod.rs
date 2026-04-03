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
