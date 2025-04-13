use crate::{ctx::Ctx, error::Result, files::joined_path, jsend};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Deserialize)]
enum UploadEntryType {
    Directory,
    File,
}

#[derive(Deserialize)]
struct UploadEntry {
    entry_type: UploadEntryType,
    name: String,
    relative_path: String,
    size: u64,
}

#[derive(Deserialize)]
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
    path: Option<Path<String>>,
    Json(payload): Json<RegisterUploadRequest>,
) -> Response {
    let rel_path = match path {
        Some(Path(p)) => PathBuf::from(p),
        // todo: jsend
        None => return (StatusCode::BAD_REQUEST, "file path required").into_response(),
    };

    // todo: for now we won't allow any sort of merging or overriding of files/folders

    // Check that path exists and is valid

    let root_path = PathBuf::from(super::ROOT_PATH);
    let Some(path) = joined_path(&root_path, &rel_path) else {
        return jsend::JSendBuilder::new()
            .fail("invalid path")
            .into_response();
    };

    let Ok(metadata) = fs::metadata(path.clone()).await else {
        return jsend::JSendBuilder::new().fail("bad path").into_response();
    };

    "test".into_response()
}
