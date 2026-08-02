use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
use jiff::Timestamp;

use crate::{guest::Guest, Result};

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GuestToken {
    pub guest_id: String,
    pub issued_at_unix: i64,
    pub expires_at_unix: i64,
}

impl GuestToken {
    fn issued_at(&self) -> Result<Timestamp> {
        Timestamp::from_second(self.issued_at_unix).with_context(|| {
            format!(
                "guest token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            )
        })
    }

    pub fn is_expired(&self) -> bool {
        Timestamp::now().as_second() > self.expires_at_unix
    }
}

impl TryFrom<&GuestToken> for Guest {
    type Error = crate::Error;

    fn try_from(token: &GuestToken) -> Result<Self> {
        Ok(Guest {
            id: token.guest_id.clone(),
            created_at: token.issued_at()?,
        })
    }
}

pub fn decode_token(key: &Key, token: &str) -> Result<GuestToken> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode guest token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("guest token is too short"));
    };
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|err| anyhow!(err).context("could not read guest token nonce"))?;

    let cipher = ChaCha20Poly1305::new(key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt guest token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode guest token payload"))
}
