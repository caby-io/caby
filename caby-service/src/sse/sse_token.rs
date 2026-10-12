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

pub const TOKEN_LIFETIME_SECS: i64 = 10 * 60;

#[derive(Encode, Decode)]
pub struct SseToken {
    pub account: String,
    pub issued_at_unix: i64,
}

impl SseToken {
    pub fn is_expired(&self) -> bool {
        let Some(expires_at_unix) = self.issued_at_unix.checked_add(TOKEN_LIFETIME_SECS) else {
            error!(
                "sse token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            );
            return true;
        };

        Timestamp::now().as_second() > expires_at_unix
    }
}

pub fn generate_token(cfg: &Config, account: &str) -> Result<(String, SseToken)> {
    let payload = SseToken {
        account: account.to_owned(),
        issued_at_unix: Timestamp::now().as_second(),
    };

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let nonce = Nonce::generate();

    let plaintext = bitcode::encode(&payload);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt sse token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok((URL_SAFE_NO_PAD.encode(&combined), payload))
}

pub fn decode_token(cfg: &Config, token: &str) -> Result<SseToken> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode sse token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("sse token is too short"));
    };
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|err| anyhow!(err).context("could not read sse token nonce"))?;

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt sse token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode sse token payload"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(issued_at_unix: i64) -> SseToken {
        SseToken {
            account: "holden".to_owned(),
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
