use crate::{config::Config, ctx::Ctx, error::Result, files::joined_path, jsend};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub enum PutEntryType {
    Directory,
    File,
    Upload,
}

#[derive(Deserialize)]
pub struct PutEntryRequest {
    pub entry_type: PutEntryType,
    pub name: String,
    // todo: this probably isnt the best for raw files?
    pub content: Option<String>,
}

// used to create directories and small, inline, files
pub async fn handle_put_files(
    State(cfg): State<Config>,
    ctx: Result<Ctx>,
    path: Option<Path<String>>,
    Json(payload): Json<PutEntryRequest>,
) -> Response {
    let rel_path = match path {
        Some(Path(p)) => PathBuf::from(p),
        // todo: jsend
        None => return (StatusCode::BAD_REQUEST, "file path required").into_response(),
    };

    let Some(path) = joined_path(&cfg.live_path, &rel_path) else {
        return jsend::JSendBuilder::new()
            .fail("invalid path")
            .into_response();
    };

    //todo
    "test".into_response()
}
