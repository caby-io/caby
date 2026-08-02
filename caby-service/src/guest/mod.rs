use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jiff::Timestamp;
use rand::RngExt;

pub mod token;

#[derive(Clone, Debug)]
pub struct Guest {
    pub id: String,
    pub created_at: Timestamp,
}

impl Guest {
    pub fn new() -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            created_at: Timestamp::now(),
        }
    }
}
