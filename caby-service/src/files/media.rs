use std::path::Path;

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
};
use chrono::{DateTime, Duration, Utc};

use crate::{config::Config, Result};

const TOKEN_LIFETIME_MINS: i64 = 60;

pub type MediaTokenStr = String;

#[derive(Encode, Decode)]
pub struct MediaToken {
    pub space: String,
    pub dir: String,
    pub issued_at_unix: i64,
}

impl MediaToken {
    pub fn is_expired(&self) -> bool {
        let issued = DateTime::from_timestamp(self.issued_at_unix, 0)
            .expect("issued_at_unix out of valid DateTime range");
        Utc::now() > issued + Duration::minutes(TOKEN_LIFETIME_MINS)
    }
}

pub fn generate_token(cfg: &Config, space: &str, dir: &str) -> Result<MediaTokenStr> {
    let payload = MediaToken {
        space: space.to_owned(),
        dir: dir.to_owned(),
        issued_at_unix: Utc::now().timestamp(),
    };

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let plaintext = bitcode::encode(&payload);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt media token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&combined))
}

pub fn decode_token(cfg: &Config, token: &str) -> Result<MediaToken> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode media token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("media token is too short"));
    };
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt media token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode media token payload"))
}

pub struct MediaUrlFactory<'a> {
    cfg: &'a Config,
    space: &'a str,
    token: String,
}

impl<'a> MediaUrlFactory<'a> {
    pub fn new(cfg: &'a Config, space: &'a str, dir: &Path) -> Result<Self> {
        let token = generate_token(cfg, space, &dir.to_string_lossy())?;
        Ok(Self { cfg, space, token })
    }

    fn endpoint_url(&self, kind: &str, rel: &Path) -> String {
        let mut url = self.cfg.urls.backend.clone();
        url.set_path(&format!(
            "/v0/files/{}/{}/{}",
            kind,
            self.space,
            rel.to_string_lossy()
        ));
        url.query_pairs_mut().append_pair("token", &self.token);
        url.to_string()
    }

    pub fn thumbnail_url_for(&self, rel: &Path) -> String {
        self.endpoint_url("thumbnail", rel)
    }

    pub fn preview_url_for(&self, rel: &Path) -> String {
        self.endpoint_url("preview", rel)
    }
}
