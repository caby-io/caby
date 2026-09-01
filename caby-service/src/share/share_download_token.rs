use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, Generate},
    ChaCha20Poly1305, KeyInit, Nonce,
};
use jiff::Timestamp;
use tracing::error;

use crate::{config::Config, Result};

const TOKEN_LIFETIME_MINS: i64 = 60;
const TOKEN_LIFETIME_SECS: i64 = TOKEN_LIFETIME_MINS * 60;

#[derive(Encode, Decode)]
pub struct DownloadToken {
    pub share_id: String,
    pub path: String,
    pub issued_at_unix: i64,
}

impl DownloadToken {
    pub fn is_expired(&self) -> bool {
        let Some(expires_at_unix) = self.issued_at_unix.checked_add(TOKEN_LIFETIME_SECS) else {
            error!(
                "download token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            );
            return true;
        };

        Timestamp::now().as_second() > expires_at_unix
    }
}

pub fn generate_token(cfg: &Config, share_id: &str, path: &str) -> Result<String> {
    let payload = DownloadToken {
        share_id: share_id.to_owned(),
        path: path.to_owned(),
        issued_at_unix: Timestamp::now().as_second(),
    };

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let nonce = Nonce::generate();

    let plaintext = bitcode::encode(&payload);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt download token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&combined))
}

pub fn decode_token(cfg: &Config, token: &str) -> Result<DownloadToken> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode download token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("download token is too short"));
    };
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|err| anyhow!(err).context("could not read download token nonce"))?;

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt download token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode download token payload"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(issued_at_unix: i64) -> DownloadToken {
        DownloadToken {
            share_id: "abc123".to_owned(),
            path: "photos/holiday/beach.jpg".to_owned(),
            issued_at_unix,
        }
    }

    #[test]
    fn fresh_token_is_not_expired() {
        assert!(!token(Timestamp::now().as_second()).is_expired());
    }

    #[test]
    fn stale_token_is_expired() {
        let issued_at_unix = Timestamp::now().as_second() - TOKEN_LIFETIME_SECS - 1;
        assert!(token(issued_at_unix).is_expired());
    }

    #[test]
    fn out_of_range_issued_at_fails_closed() {
        assert!(token(i64::MAX).is_expired());
    }
}
