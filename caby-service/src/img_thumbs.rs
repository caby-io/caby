use std::path::{Path, PathBuf};

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
};
use chrono::{DateTime, Duration, Utc};
use libvips::ops;
use tokio::{fs, task};

use crate::{
    config::Config,
    space::{Space, SpaceDir},
    Result,
};

pub const IMG_THUMB_FILENAME: &str = "thumb.webp";
const THUMB_TOKEN_LIFETIME_MINS: i64 = 60;

const IMG_MIMES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/heic",
    "image/heif",
    "image/avif",
];

#[derive(Debug)]
pub enum ThumbError {
    UnsupportedFormat,
    Other(anyhow::Error),
}

impl std::fmt::Display for ThumbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbError::UnsupportedFormat => write!(f, "unsupported format"),
            ThumbError::Other(err) => write!(f, "{:#}", err),
        }
    }
}

impl std::error::Error for ThumbError {}

pub fn thumb_path(space: &Space, rel: &Path) -> Result<PathBuf> {
    Ok(space.join(SpaceDir::META, rel)?.join(IMG_THUMB_FILENAME))
}

pub type ThumbToken = String;

#[derive(Encode, Decode)]
pub struct ThumbTokenPayload {
    pub space: String,
    pub dir: String,
    pub issued_at_unix: i64,
}

impl ThumbTokenPayload {
    pub fn is_expired(&self) -> bool {
        let issued = DateTime::from_timestamp(self.issued_at_unix, 0)
            .expect("issued_at_unix out of valid DateTime range");
        Utc::now() > issued + Duration::minutes(THUMB_TOKEN_LIFETIME_MINS)
    }
}

pub fn generate_thumb_token(cfg: &Config, space: &str, dir: &str) -> Result<ThumbToken> {
    let payload = ThumbTokenPayload {
        space: space.to_owned(),
        dir: dir.to_owned(),
        issued_at_unix: Utc::now().timestamp(),
    };

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let plaintext = bitcode::encode(&payload);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt thumb token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&combined))
}

pub fn decode_thumb_token(cfg: &Config, token: &str) -> Result<ThumbTokenPayload> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode thumb token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("thumb token is too short"));
    };
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt thumb token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode thumb token payload"))
}

pub fn thumb_url(cfg: &Config, space: &str, rel: &Path, token: &str) -> String {
    let mut url = cfg.urls.backend.clone();
    url.set_path(&format!(
        "/v0/files/thumbnail/{}/{}",
        space,
        rel.to_string_lossy()
    ));
    url.query_pairs_mut().append_pair("token", token);
    url.to_string()
}

pub struct ThumbUrlBuilder<'a> {
    cfg: &'a Config,
    space: &'a str,
    token: String,
}

impl<'a> ThumbUrlBuilder<'a> {
    pub fn new(cfg: &'a Config, space: &'a str, dir: &Path) -> Result<Self> {
        let token = generate_thumb_token(cfg, space, &dir.to_string_lossy())?;
        Ok(Self { cfg, space, token })
    }

    pub fn url_for(&self, rel: &Path) -> String {
        thumb_url(self.cfg, self.space, rel, &self.token)
    }
}

// todo: Put into task/queue system
pub async fn try_generate_thumb(
    live_path: &Path,
    thumb_path: &Path,
    max_edge: u32,
) -> std::result::Result<(), ThumbError> {
    let bytes = fs::read(live_path)
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("read source image")))?;

    let kind = infer::get(&bytes).ok_or(ThumbError::UnsupportedFormat)?;
    if !IMG_MIMES.contains(&kind.mime_type()) {
        return Err(ThumbError::UnsupportedFormat);
    }

    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| ThumbError::Other(anyhow!(err).context("mkdir thumb parent")))?;
    }

    // Unique tmp name so concurrent generators never collide on the same .tmp file.
    let tmp_path = thumb_path.with_extension(format!("webp.tmp.{}", xid::new()));
    let tmp_str = tmp_path
        .to_str()
        .ok_or_else(|| ThumbError::Other(anyhow!("non-UTF8 thumb tmp path")))?
        .to_owned();

    let width = max_edge as i32;
    let blocking = task::spawn_blocking(move || -> std::result::Result<(), String> {
        // thumbnail_buffer decodes, resizes within max width, applies EXIF orientation
        // (no_rotate defaults to false), all in one streaming op.
        let img = ops::thumbnail_buffer(&bytes, width)
            .map_err(|e| format!("libvips thumbnail: {:?}", e))?;
        ops::webpsave(&img, &tmp_str).map_err(|e| format!("libvips webpsave: {:?}", e))?;
        Ok(())
    })
    .await
    .map_err(|err| ThumbError::Other(anyhow!(err).context("thumbnail task panicked")))?;

    if let Err(msg) = blocking {
        // Clean up the tmp file on failure; ignore errors from the cleanup itself.
        let _ = fs::remove_file(&tmp_path).await;
        return Err(ThumbError::Other(anyhow!(msg)));
    }

    fs::rename(&tmp_path, thumb_path)
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("publish thumb via rename")))?;

    Ok(())
}
