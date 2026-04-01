use crate::{
    config::Config,
    jsend::JSendBuilder,
    space::{Space, SpaceDir},
};
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Deserialize)]
pub struct MoveEntriesRequest {
    pub entries: Vec<(String, String)>,
    pub force: bool,
}

#[derive(Serialize)]
struct MoveEntriesResponse {
    pub moved: Vec<(String, String)>,
    pub errors: Vec<MoveError>,
}

#[derive(Serialize)]
struct MoveError {
    pub src: String,
    pub dst: String,
    pub error: String,
}

impl MoveError {
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
    space: Space,
    Json(req): Json<MoveEntriesRequest>,
) -> Response {
    let mut moved = vec![];
    let mut errors = vec![];

    for (input_src, input_dst) in req.entries {
        // Build & validate source path
        let src_rpath = PathBuf::from(input_src.clone()).clean();
        let Ok(src_path) = space.join(SpaceDir::LIVE, &src_rpath) else {
            errors.push(MoveError::new(input_src, input_dst, "invalid source"));
            continue;
        };

        let Ok(src_metadata) = fs::metadata(src_path.clone()).await else {
            errors.push(MoveError::new(input_src, input_dst, "source not found"));
            continue;
        };

        // Build & validate destination path
        let dst_rpath = PathBuf::from(input_dst.clone()).clean();
        let Ok(dst_path) = space.join(SpaceDir::LIVE, &dst_rpath) else {
            errors.push(MoveError::new(input_src, input_dst, "invalid destination"));
            continue;
        };

        let Ok(exists) = fs::try_exists(dst_path.clone()).await else {
            // todo: log base error
            errors.push(MoveError::new(
                input_src,
                input_dst,
                "could not check if destination exists",
            ));
            continue;
        };

        if exists {
            errors.push(MoveError::new(input_src, input_dst, "destination exists"));
            continue;
        }

        if let Err(err) = fs::rename(src_path, dst_path).await {
            errors.push(MoveError::new(
                input_src,
                input_dst,
                format!("could not move: {}", err),
            ));
            continue;
        }

        moved.push((
            src_rpath.to_str().unwrap().to_owned(),
            dst_rpath.to_str().unwrap().to_owned(),
        ));
    }

    JSendBuilder::new()
        .success(MoveEntriesResponse { moved, errors })
        .into_response()
}
