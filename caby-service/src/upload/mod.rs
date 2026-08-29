use std::{os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::{anyhow, Context};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, Generate},
    ChaCha20Poly1305, KeyInit, Nonce,
};
use jiff::Timestamp;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use tracing::error;
use xxhash_rust::xxh64::Xxh64;

use crate::{config::Config, Result};

pub mod manifest;

pub const UPLOAD_TOKEN_LIFETIME_HOURS: i64 = 24;
const UPLOAD_TOKEN_LIFETIME_SECS: i64 = UPLOAD_TOKEN_LIFETIME_HOURS * 3600;

#[derive(Encode, Decode)]
pub struct UploadTokenPayload {
    pub id: String,
    pub issued_at_unix: i64,
    pub base_path: String,
    pub chunk_size: u64,
    pub total_size: u64,
    // todo: have two modes for payload:
    //   1. short file list: encode inline in the token
    //   2. long file list: encode just the total size, then validate per-file on
    //       completion (so the user can't burn all the space)
}

impl UploadTokenPayload {
    pub fn issued_at(&self) -> Result<Timestamp> {
        Timestamp::from_second(self.issued_at_unix).with_context(|| {
            format!(
                "upload token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            )
        })
    }

    pub fn is_expired(&self) -> bool {
        let Some(expires_at_unix) = self.issued_at_unix.checked_add(UPLOAD_TOKEN_LIFETIME_SECS)
        else {
            error!(
                "upload token issued_at_unix out of valid range: {}",
                self.issued_at_unix
            );
            return true;
        };

        Timestamp::now().as_second() > expires_at_unix
    }
}

pub type UploadToken = String;

pub fn generate_upload_token(cfg: &Config, payload: UploadTokenPayload) -> Result<UploadToken> {
    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let nonce = Nonce::generate();

    let plaintext = bitcode::encode(&payload);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|err| anyhow!(err).context("could not encrypt upload token"))?;

    let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);

    Ok(URL_SAFE_NO_PAD.encode(&combined))
}

pub fn decode_upload_token(cfg: &Config, token: &str) -> Result<UploadTokenPayload> {
    let combined = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|err| anyhow!(err).context("could not base64-decode upload token"))?;

    let Some((nonce_bytes, ciphertext)) = combined.split_at_checked(12) else {
        return Err(anyhow!("upload token is too short"));
    };
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|err| anyhow!(err).context("could not read upload token nonce"))?;

    let cipher = ChaCha20Poly1305::new(&cfg.token_encryption_key);
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| anyhow!(err).context("could not decrypt upload token"))?;

    bitcode::decode(&plaintext)
        .map_err(|err| anyhow!(err).context("could not decode upload token payload"))
}

pub async fn get_file_digest_size(file_path: PathBuf) -> Result<(String, u64)> {
    let file = match File::open(file_path).await {
        Ok(f) => f,
        Err(err) => return Err(err.into()),
    };

    let size = match file.metadata().await {
        Ok(m) => m.size(),
        Err(err) => return Err(err.into()),
    };

    let mut buf_reader = BufReader::new(file);
    let mut buffer = [0; 1024]; // todo: variablize size
    let mut hash = Xxh64::new(0); // todo: variablize seed
    loop {
        let count = match buf_reader.read(&mut buffer).await {
            Ok(c) => c,
            Err(err) => return Err(err.into()),
        };
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }

    // todo: padding
    Ok((format!("{:0>16x}", hash.digest()), size))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(issued_at_unix: i64) -> UploadTokenPayload {
        UploadTokenPayload {
            id: "upl0ad".to_owned(),
            issued_at_unix,
            base_path: "rocinante".to_owned(),
            chunk_size: 1024,
            total_size: 4096,
        }
    }

    #[test]
    fn fresh_token_is_not_expired() {
        assert!(!payload(Timestamp::now().as_second()).is_expired());
    }

    #[test]
    fn stale_token_is_expired() {
        let issued_at_unix = Timestamp::now().as_second() - UPLOAD_TOKEN_LIFETIME_SECS - 1;
        assert!(payload(issued_at_unix).is_expired());
    }

    #[test]
    fn out_of_range_issued_at_fails_closed() {
        assert!(payload(i64::MAX).is_expired());
        assert!(payload(i64::MAX).issued_at().is_err());
    }
}
