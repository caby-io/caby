use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use rand::RngExt;

pub mod token;

#[derive(Clone, Debug)]
pub struct ShareAccess {
    pub space: String,
    pub id: String,
    pub password_fingerprint: Option<u64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct Guest {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub share_access: Vec<ShareAccess>,
}

impl Guest {
    pub fn new() -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            created_at: Utc::now(),
            share_access: Vec::new(),
        }
    }

    pub fn access_for(&self, space: &str, share_id: &str) -> Option<&ShareAccess> {
        self.share_access
            .iter()
            .find(|access| access.space == space && access.id == share_id)
    }
}
