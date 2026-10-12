use std::path::Path;

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, KeyInit, Mac};
use jiff::Timestamp;
use sha2::Sha256;

use crate::{config::Config, web::v0_path, Result};

const ROUND_SECS: i64 = 60 * 60;
const LIFETIME_SECS: i64 = 2 * ROUND_SECS;
const HMAC_TAG_LEN: usize = 16;
const MEDIA_KEY_LABEL: &[u8] = b"caby/media-token/v1";

type HmacSha256 = Hmac<Sha256>;

pub type MediaTokenStr = String;

fn new_mac(key: &[u8]) -> HmacSha256 {
    HmacSha256::new_from_slice(key).expect("hmac accepts keys of any length")
}

struct MediaTokenClaim<'a> {
    space: &'a str,
    dir: &'a str,
    issued_at: i64,
}

impl MediaTokenClaim<'_> {
    fn mac(&self, token_key: &[u8]) -> HmacSha256 {
        let mut derive = new_mac(token_key);
        derive.update(MEDIA_KEY_LABEL);

        let mut mac = new_mac(&derive.finalize().into_bytes());
        for field in [self.space.as_bytes(), self.dir.as_bytes()] {
            mac.update(&(field.len() as u64).to_be_bytes());
            mac.update(field);
        }
        mac.update(&self.issued_at.to_be_bytes());
        mac
    }

    fn sign(&self, token_key: &[u8]) -> MediaTokenStr {
        let tag = self.mac(token_key).finalize().into_bytes();

        let mut combined = Vec::with_capacity(size_of::<i64>() + HMAC_TAG_LEN);
        combined.extend_from_slice(&self.issued_at.to_be_bytes());
        combined.extend_from_slice(&tag[..HMAC_TAG_LEN]);

        URL_SAFE_NO_PAD.encode(&combined)
    }
}

fn try_verify_at(token_key: &[u8], space: &str, dir: &str, token: &str, now: i64) -> Result<i64> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode media token"))?;

    let Some((issued_at_bytes, tag)) = combined.split_first_chunk::<{ size_of::<i64>() }>() else {
        return Err(anyhow!("media token is too short"));
    };
    if tag.len() != HMAC_TAG_LEN {
        return Err(anyhow!("media token has the wrong length"));
    }

    let claim = MediaTokenClaim {
        space,
        dir,
        issued_at: i64::from_be_bytes(*issued_at_bytes),
    };
    if claim.issued_at > now {
        return Err(anyhow!("media token is from the future"));
    }
    let expires_at = claim.issued_at + LIFETIME_SECS;
    if now >= expires_at {
        return Err(anyhow!("media token is expired"));
    }

    claim
        .mac(token_key)
        .verify_truncated_left(tag)
        .map_err(|_| anyhow!("media token signature mismatch"))?;

    Ok(expires_at)
}

pub fn try_verify_file(cfg: &Config, space: &str, rel_path: &Path, token: &str) -> Result<i64> {
    let dir = rel_path
        .parent()
        .map(|p| p.to_string_lossy())
        .unwrap_or_default();

    try_verify_at(
        cfg.token_encryption_key.as_slice(),
        space,
        &dir,
        token,
        Timestamp::now().as_second(),
    )
}

pub struct MediaUrlFactory<'a> {
    cfg: &'a Config,
    space: &'a str,
    token: MediaTokenStr,
}

impl<'a> MediaUrlFactory<'a> {
    pub fn new(cfg: &'a Config, space: &'a str, dir: &Path) -> Self {
        let now = Timestamp::now().as_second();
        let token = MediaTokenClaim {
            space,
            dir: &dir.to_string_lossy(),
            issued_at: now - now % ROUND_SECS,
        }
        .sign(cfg.token_encryption_key.as_slice());
        Self { cfg, space, token }
    }

    fn endpoint_url(&self, kind: &str, rel: &Path, version: i64) -> String {
        let mut url = self.cfg.urls.backend.clone();
        url.set_path(&format!(
            "{}/files/{}/{}/{}",
            v0_path(self.cfg.urls.backend.path()),
            kind,
            self.space,
            rel.to_string_lossy()
        ));
        url.query_pairs_mut()
            .append_pair("token", &self.token)
            .append_pair("v", &version.to_string());
        url.to_string()
    }

    pub fn thumbnail_url_for(&self, rel: &Path, version: i64) -> String {
        self.endpoint_url("thumbnail", rel, version)
    }

    pub fn preview_url_for(&self, rel: &Path, version: i64) -> String {
        self.endpoint_url("preview", rel, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = &[7; 32];
    const NOW: i64 = 490_000 * ROUND_SECS + 1_800;
    const ISSUED_AT: i64 = NOW - NOW % ROUND_SECS;

    fn claim<'a>(space: &'a str, dir: &'a str, issued_at: i64) -> MediaTokenClaim<'a> {
        MediaTokenClaim {
            space,
            dir,
            issued_at,
        }
    }

    fn sign(space: &str, dir: &str, issued_at: i64) -> MediaTokenStr {
        claim(space, dir, issued_at).sign(KEY)
    }

    fn verify(space: &str, dir: &str, token: &str) -> Result<i64> {
        try_verify_at(KEY, space, dir, token, NOW)
    }

    #[test]
    fn same_inputs_give_same_token() {
        assert_eq!(
            sign("rocinante", "holiday", ISSUED_AT),
            sign("rocinante", "holiday", ISSUED_AT)
        );
    }

    #[test]
    fn current_token_verifies() {
        let token = sign("rocinante", "holiday", ISSUED_AT);
        assert_eq!(
            verify("rocinante", "holiday", &token).ok(),
            Some(ISSUED_AT + LIFETIME_SECS)
        );
    }

    #[test]
    fn token_expires_at_lifetime_boundary() {
        let token = sign("rocinante", "holiday", ISSUED_AT);
        let expires_at = ISSUED_AT + LIFETIME_SECS;
        assert!(try_verify_at(KEY, "rocinante", "holiday", &token, expires_at - 1).is_ok());
        assert!(try_verify_at(KEY, "rocinante", "holiday", &token, expires_at).is_err());
    }

    #[test]
    fn previous_hour_token_verifies() {
        let token = sign("rocinante", "holiday", ISSUED_AT - ROUND_SECS);
        assert!(verify("rocinante", "holiday", &token).is_ok());
    }

    #[test]
    fn stale_token_is_rejected() {
        let token = sign("rocinante", "holiday", ISSUED_AT - LIFETIME_SECS);
        assert!(verify("rocinante", "holiday", &token).is_err());
    }

    #[test]
    fn future_token_is_rejected() {
        let token = sign("rocinante", "holiday", ISSUED_AT + ROUND_SECS);
        assert!(verify("rocinante", "holiday", &token).is_err());
    }

    #[test]
    fn out_of_range_issued_at_fails_closed() {
        assert!(verify(
            "rocinante",
            "holiday",
            &sign("rocinante", "holiday", i64::MIN)
        )
        .is_err());
        assert!(verify(
            "rocinante",
            "holiday",
            &sign("rocinante", "holiday", i64::MAX)
        )
        .is_err());
    }

    #[test]
    fn wrong_space_or_dir_is_rejected() {
        let token = sign("rocinante", "holiday", ISSUED_AT);
        assert!(verify("tachi", "holiday", &token).is_err());
        assert!(verify("rocinante", "holiday/ceres", &token).is_err());
    }

    #[test]
    fn ambiguous_field_split_is_rejected() {
        let token = sign("roci", "nanteholiday", ISSUED_AT);
        assert!(verify("rocinante", "holiday", &token).is_err());
    }

    #[test]
    fn wrong_key_is_rejected() {
        let token = claim("rocinante", "holiday", ISSUED_AT).sign(&[8; 32]);
        assert!(verify("rocinante", "holiday", &token).is_err());
    }

    #[test]
    fn tampered_tag_is_rejected() {
        let token = sign("rocinante", "holiday", ISSUED_AT);
        let mut bytes = URL_SAFE_NO_PAD.decode(&token).unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        assert!(verify("rocinante", "holiday", &URL_SAFE_NO_PAD.encode(&bytes)).is_err());
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        let token = sign("rocinante", "holiday", ISSUED_AT);
        assert!(verify("rocinante", "holiday", &token[..token.len() - 2]).is_err());
        assert!(verify("rocinante", "holiday", "AAAA").is_err());
        assert!(verify("rocinante", "holiday", "not base64!").is_err());
    }
}
