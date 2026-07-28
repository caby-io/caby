use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, Generate},
    ChaCha20Poly1305, Key, KeyInit, Nonce,
};
use chrono::{DateTime, Duration, Utc};

use crate::{
    guest::{Guest, ShareAccess},
    Result,
};

pub const DEFAULT_GUEST_TOKEN_LIFETIME_DAYS: i64 = 7;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GuestGrant {
    pub space: String,
    pub share_id: String,
    pub pw_fingerprint: u64,
}

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct GuestToken {
    pub guest_id: String,
    pub issued_at_unix: i64,
    pub expires_at_unix: i64,
    pub grants: Vec<GuestGrant>,
}

impl GuestToken {
    pub fn new(guest_id: &str, grants: Vec<GuestGrant>, lifetime: Duration) -> Self {
        let now = Utc::now();
        Self {
            guest_id: guest_id.to_owned(),
            issued_at_unix: now.timestamp(),
            expires_at_unix: (now + lifetime).timestamp(),
            grants,
        }
    }

    fn issued_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.issued_at_unix, 0)
            .expect("issued_at_unix out of valid DateTime range")
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.expires_at_unix, 0)
            .expect("expires_at_unix out of valid DateTime range")
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.expires_at_unix
    }
}

impl From<&GuestToken> for Guest {
    fn from(token: &GuestToken) -> Self {
        Guest {
            id: token.guest_id.clone(),
            created_at: token.issued_at(),
            share_access: token
                .grants
                .iter()
                .map(|grant| ShareAccess {
                    space: grant.space.clone(),
                    id: grant.share_id.clone(),
                    password_fingerprint: Some(grant.pw_fingerprint),
                    created_at: token.issued_at(),
                })
                .collect(),
        }
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
        GuestToken::new(
            "guest-abc",
            vec![GuestGrant {
                space: "home".to_owned(),
                share_id: "share-abc".to_owned(),
                pw_fingerprint: 0x1234_5678_9abc_def0,
            }],
            Duration::days(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS),
        )
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

        let stale = GuestToken::new("guest-abc", vec![], Duration::days(-1));
        assert!(stale.is_expired());
    }

    #[test]
    fn decodes_into_guest_with_grants() {
        let token = sample();
        let guest: Guest = (&token).into();
        assert_eq!(guest.id, "guest-abc");
        assert_eq!(guest.share_access.len(), 1);
        assert_eq!(guest.share_access[0].space, "home");
        assert_eq!(guest.share_access[0].id, "share-abc");
        assert_eq!(
            guest.share_access[0].password_fingerprint,
            Some(0x1234_5678_9abc_def0)
        );
    }
}
