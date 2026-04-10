use std::path::PathBuf;

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::AuthorizedUser,
    config::Config,
    files::overview::{build_overview, OverviewEntry},
    jsend,
    space::Space,
    web::files_api::files_list::FilesPathParams,
};

// #[derive(Deserialize)]
// pub struct SummarizeFilesRequest {}

#[derive(Deserialize)]
pub struct FilesOverviewParams {
    pub dirs_only: Option<bool>,
}

#[derive(Serialize)]
struct SummarizeFilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub entries: Vec<OverviewEntry>,
}

pub async fn handle_files_overview(
    State(cfg): State<Config>,
    space: Space,
    user: AuthorizedUser,
    path_params: Path<FilesPathParams>,
    Query(params): Query<FilesOverviewParams>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    // todo: sanitize path, more
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), |p| PathBuf::from(p));

    // let Ok(path) = space.join(&rel_path) else {
    //     return resp.fail("invalid path").into_response();
    // };

    let entries =
        match build_overview(&space, &rel_path, 20, params.dirs_only.unwrap_or(false)).await {
            Ok(r) => r,
            Err(err) => {
                return resp
                    // todo: don't send this down in production, just log the actual error
                    .error(format!("could not access files: {}", err))
                    .into_response();
            }
        };
    // let Ok((dirs, files)) = get_entries(PathBuf::from("/").join(&path).clean().as_path()).await
    // else {
    //     return resp.error("could not access files: {}").into_response();
    // };

    // todo: make safe
    let parent_dir = rel_path.parent().map(|p| p.to_str().unwrap().to_owned());

    jsend::JSendBuilder::new()
        .success(SummarizeFilesResponse {
            path: rel_path.to_str().unwrap().to_owned(), // todo: make safe
            parent_dir,
            entries,
        })
        .into_response()
}
