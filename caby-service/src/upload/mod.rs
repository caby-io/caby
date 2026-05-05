use std::{os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitcode::{Decode, Encode};
use chacha20poly1305::{
    aead::{Aead, OsRng},
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
};
use chrono::{DateTime, Duration, Utc};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use xxhash_rust::xxh64::Xxh64;

use crate::{config::Config, Result};

pub mod manifest;

const UPLOAD_TOKEN_LIFETIME_HOURS: i64 = 24;

#[derive(Encode, Decode)]
pub struct UploadTokenPayload {
    pub id: String,
    pub issued_at_unix: i64,
    pub base_path: String,
    pub chunk_size: u64,
    pub total_size: u64,
    // TODO: validate the file list without always loading it. Two modes:
    //   (a) short list — encode inline in the token
    //   (b) long list — encode just the total size, then validate per-file on
    //       completion (so the user can't burn all the space)
}

impl UploadTokenPayload {
    pub fn issued_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.issued_at_unix, 0)
            .expect("issued_at_unix out of valid DateTime range")
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.issued_at() + Duration::hours(UPLOAD_TOKEN_LIFETIME_HOURS)
    }
}

pub type UploadToken = String;

pub fn generate_upload_token(cfg: &Config, payload: UploadTokenPayload) -> Result<UploadToken> {
    let cipher = ChaCha20Poly1305::new(&cfg.upload_token_key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

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
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = ChaCha20Poly1305::new(&cfg.upload_token_key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
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
