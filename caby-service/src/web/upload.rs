use std::{os::unix::fs::MetadataExt, path::PathBuf, sync::Arc};

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::prelude::*;
use bitcode::{Decode, Encode};
use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use xxhash_rust::xxh64::Xxh64;

use crate::{error, jsend::JSendBuilder, Error};

// todo: move some of these fn's out of the web dir

// Upload registration

pub static MAX_CHUNK_SIZE: u64 = 10_000_000;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum UploadEntryType {
    Directory,
    File,
}

// todo: need to consider how best to handle empty dirs
#[derive(Deserialize, Debug)]
pub struct UploadEntry {
    pub entry_type: UploadEntryType,
    pub name: String,
    pub size: Option<u64>,
    pub xxh_digest: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    Override,
    Skip,
    Prompt,
    Deconflict,
}

// Upload tokens

pub const HEADER_UPLOAD_TOKEN: &str = "Caby-Upload-Token";
pub const HEADER_CHUNK_INDEX: &str = "Caby-Chunk-Index";

pub type UploadToken = String;

#[derive(Encode, Decode, Debug)]
pub struct TokenFile {
    pub name: String,
    pub size: Option<u64>,
}

#[derive(Encode, Decode, Debug)]
pub struct UploadTokenPayload {
    pub id: String,
    pub chunk_size: u64,
    pub files: Vec<TokenFile>,
}

impl Into<UploadToken> for UploadTokenPayload {
    fn into(self) -> UploadToken {
        BASE64_STANDARD.encode(&bitcode::encode(&self))
    }
}

pub struct UploadTokenParseError {}

impl IntoResponse for UploadTokenParseError {
    fn into_response(self) -> Response {
        JSendBuilder::new()
            .status_code(StatusCode::BAD_REQUEST)
            .fail("bad upload token")
            .into_response()
    }
}

impl TryInto<UploadTokenPayload> for UploadToken {
    type Error = UploadTokenParseError;

    fn try_into(self) -> Result<UploadTokenPayload, Self::Error> {
        let bytes = BASE64_STANDARD
            .decode(self.as_bytes())
            .map_err(|_| UploadTokenParseError {})?;
        bitcode::decode(&bytes).map_err(|_| UploadTokenParseError {})
    }
}

pub fn u64_to_b64(n: u64) -> String {
    BASE64_STANDARD.encode(&n.to_ne_bytes())
}

pub async fn get_file_digest_size(file_path: PathBuf) -> error::Result<(String, u64)> {
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

// pub fn get_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
//     headers.get(key)?.to_str().ok().map(|s| s.to_owned())
// }

// pub fn get_upload_token_header(headers: &HeaderMap) -> Option<UploadTokenPayload> {
//     let header_value = get_header_value(headers, HEADER_UPLOAD_TOKEN)?;
//     header_value.try_into().ok()
// }
