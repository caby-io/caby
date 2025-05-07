use crate::{ctx::Ctx, error::Result, jsend};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

#[derive(Deserialize, Debug)]
enum UploadEntryType {
    Directory,
    File,
}

// todo: need to consider how best to handle empty dirs
#[derive(Deserialize, Debug)]
pub struct UploadEntry {
    entry_type: UploadEntryType,
    name: String,
    dest: String,
    size: u64,
}

#[derive(Deserialize, Debug)]
pub struct RegisterUploadRequest {
    pub entries: Vec<UploadEntry>,
    // todo: additional controls such as overriding?
}

#[derive(Serialize)]
struct RegisterUploadResponse {
    pub id: String,
    pub chunk_size: u32,
}

pub async fn handle_register_upload(
    ctx: Result<Ctx>,
    Json(payload): Json<RegisterUploadRequest>,
) -> Response {
    debug!("{:?}", payload);

    // Validate?
    // Generate an ID for this request
    let id = xid::new();
    // Create a tmp dir for this upload

    // Create a tmp file for this upload

    jsend::JSendBuilder::new()
        .success(RegisterUploadResponse {
            id: id.to_string(),
            chunk_size: 1_000_000,
        })
        .into_response()
}
