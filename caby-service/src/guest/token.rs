use std::time::Duration;

use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, Generate},
    ChaCha20Poly1305, Key, KeyInit, Nonce,
};
use jiff::Timestamp;

use crate::{guest::Guest, Result};

const DEFAULT_GUEST_TOKEN_LIFETIME_DAYS: u64 = 7;
pub const DEFAULT_GUEST_TOKEN_LIFETIME: Duration =
    Duration::from_mins(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS * 24 * 60);

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GuestToken {
    pub guest_id: String,
    pub issued_at_unix: i64,
    pub expires_at_unix: i64,
}

impl GuestToken {
    pub fn new(guest_id: &str, lifetime: Duration) -> Result<Self> {
        let now = Timestamp::now();
        let expires_at = now
            .checked_add(lifetime)
            .context("guest token expiry is out of valid range")?;

        Ok(Self {
            guest_id: guest_id.to_owned(),
            issued_at_unix: now.as_second(),
            expires_at_unix: expires_at.as_second(),
        })
    }

    fn issued_at(&self) -> Result<Timestamp> {
        Timestamp::from_second(self.issued_at_unix).with_context(|| {
            format!(
                "guest token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            )
        })
    }

    pub fn expires_at(&self) -> Result<Timestamp> {
        Timestamp::from_second(self.expires_at_unix).with_context(|| {
            format!(
                "guest token expires_at_unix out of valid range: {}",
                self.expires_at_unix
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

pub fn encode_token(key: &Key, token: &GuestToken) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::generate();

    let plaintext = bitcode::encode(token);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt guest token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&combined))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GuestToken {
        GuestToken::new("guest-abc", DEFAULT_GUEST_TOKEN_LIFETIME).unwrap()
    }

    #[test]
    fn round_trip_preserves_payload() {
        let key = Key::generate();
        let token = sample();
        let encoded = encode_token(&key, &token).unwrap();
        let decoded = decode_token(&key, &encoded).unwrap();
        assert_eq!(decoded, token);
    }

    #[test]
    fn wrong_key_fails_to_decode() {
        let key = Key::generate();
        let other = Key::generate();
        let encoded = encode_token(&key, &sample()).unwrap();
        assert!(decode_token(&other, &encoded).is_err());
    }

    #[test]
    fn tampered_token_fails_to_decode() {
        let key = Key::generate();
        let encoded = encode_token(&key, &sample()).unwrap();
        let mut bytes = URL_SAFE_NO_PAD.decode(&encoded).unwrap();
        *bytes.last_mut().unwrap() ^= 0xff;
        let tampered = URL_SAFE_NO_PAD.encode(&bytes);
        assert!(decode_token(&key, &tampered).is_err());
    }

    #[test]
    fn expiry_reflects_lifetime() {
        let fresh = sample();
        assert!(!fresh.is_expired());

        let now = Timestamp::now().as_second();
        let stale = GuestToken {
            guest_id: "guest-abc".to_owned(),
            issued_at_unix: now - 10,
            expires_at_unix: now - 1,
        };
        assert!(stale.is_expired());
    }

    #[test]
    fn decodes_into_guest() {
        let token = sample();
        let guest = Guest::try_from(&token).unwrap();
        assert_eq!(guest.id, "guest-abc");
    }
}
