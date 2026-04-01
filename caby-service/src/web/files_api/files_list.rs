use crate::{
    config::Config,
    files::{build_entries, Entry},
    jsend::JSendBuilder,
    space::Space,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const FILE_NOT_FOUND: &'static str = "file not found";

#[derive(Deserialize)]
pub struct FilesPathParams {
    pub space: String,
    pub file_path: Option<String>,
}

#[derive(Serialize)]
struct ListFilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub entries: Vec<Entry>,
}

pub async fn handle_list_files(
    State(cfg): State<Config>,
    space: Space,
    path_params: Path<FilesPathParams>,
) -> Response {
    let resp = JSendBuilder::new();
    // todo: sanitize path, more

    // temp
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), |p| PathBuf::from(p));

    // let Ok(path) = space.join(&rel_path) else {
    //     return resp.fail("invalid path").into_response();
    // };

    // todo: consider santizing after join
    // todo: check that it is a dir? OR return something else for files
    let entries = match build_entries(&space, &rel_path).await {
        Ok(r) => r,
        Err(err) => {
            return resp
                // todo: don't send this down in production, just log the actual error
                .error(format!("could not access files: {}", err))
                .into_response();
        }
    };

    // todo: make safe
    let parent_dir = rel_path.parent().map(|p| p.to_str().unwrap().to_owned());

    resp.success(ListFilesResponse {
        path: rel_path.to_str().unwrap().to_owned(), // todo: make safe
        parent_dir,
        entries,
    })
    .into_response()
}
