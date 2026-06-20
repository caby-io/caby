use crate::{auth::AuthorizedUser, config::Config, files, jsend::JSendBuilder, space::Space};
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    user: AuthorizedUser,
    Json(req): Json<MoveEntriesRequest>,
) -> Response {
    let mut moved = vec![];
    let mut errors = vec![];

    for (input_src, input_dst) in req.entries {
        let src_rpath = PathBuf::from(input_src.clone()).clean();
        let dst_rpath = PathBuf::from(input_dst.clone()).clean();

        if let Err(err) = files::ops::rename(&space, &src_rpath, &dst_rpath).await {
            errors.push(MoveError::new(input_src, input_dst, format!("{:#}", err)));
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
