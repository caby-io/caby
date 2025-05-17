use crate::{config::Config, ctx::Ctx, error::Result, files::joined_path, jsend};
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Deserialize)]
pub struct RenamedEntriesRequest {
    pub entries: Vec<(String, String)>,
    pub force: bool,
}

#[derive(Serialize)]
struct RenamedEntriesResponse {
    pub renamed: Vec<(String, String)>,
    pub errors: Vec<RenameError>,
}

#[derive(Serialize)]
struct RenameError {
    pub src: String,
    pub dst: String,
    pub error: String,
}

impl RenameError {
    pub fn new(src: impl Into<String>, dst: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            dst: dst.into(),
            error: error.into(),
        }
    }
}

pub async fn handle_move_files(
    State(cfg): State<Config>,
    ctx: Result<Ctx>,
    Json(req): Json<RenamedEntriesRequest>,
) -> Response {
    let mut renamed = vec![];
    let mut errors = vec![];

    for (input_src, input_dst) in req.entries {
        // Build & validate source path
        let src_rpath = PathBuf::from(input_src.clone()).clean();
        let Some(src_path) = joined_path(&cfg.live_path, &src_rpath) else {
            errors.push(RenameError::new(input_src, input_dst, "invalid source"));
            continue;
        };

        let Ok(src_metadata) = fs::metadata(src_path.clone()).await else {
            errors.push(RenameError::new(input_src, input_dst, "source not found"));
            continue;
        };

        // Build & validate destination path
        let dst_rpath = PathBuf::from(input_dst.clone()).clean();
        let Some(dst_path) = joined_path(&cfg.live_path, &dst_rpath) else {
            errors.push(RenameError::new(
                input_src,
                input_dst,
                "invalid destination",
            ));
            continue;
        };

        let Ok(exists) = fs::try_exists(dst_path.clone()).await else {
            // todo: log base error
            errors.push(RenameError::new(
                input_src,
                input_dst,
                "could not check if destination exists",
            ));
            continue;
        };

        if exists {
            errors.push(RenameError::new(input_src, input_dst, "destination exists"));
            continue;
        }

        if let Err(err) = fs::rename(src_path, dst_path).await {
            errors.push(RenameError::new(
                input_src,
                input_dst,
                format!("could not rename: {}", err),
            ));
            continue;
        }

        renamed.push((
            src_rpath.to_str().unwrap().to_owned(),
            dst_rpath.to_str().unwrap().to_owned(),
        ));
    }

    jsend::JSendBuilder::new()
        .success(RenamedEntriesResponse { renamed, errors })
        .into_response()
}
