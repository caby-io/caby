use crate::{
    config::Config,
    ctx::Ctx,
    error::Result,
    files::{build_entries, joined_path, Entry},
    jsend,
    space::Space,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct ListFilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub entries: Vec<Entry>,
}

pub async fn handle_list_files(
    State(cfg): State<Config>,
    ctx: Result<Ctx>,
    space: Space,
    files_path: Option<Path<String>>,
) -> Response {
    // todo: sanitize path, more
    let rel_path = files_path.map_or(PathBuf::from(""), |Path(p)| PathBuf::from(p));

    let Ok(path) = space.join(&rel_path) else {
        return jsend::JSendBuilder::new()
            .fail("invalid path")
            .into_response();
    };

    let resp = jsend::JSendBuilder::new();

    // todo: consider santizing after join
    // todo: check that it is a dir? OR return something else for files
    let entries = match build_entries(&cfg.live_path, &path).await {
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
        .success(ListFilesResponse {
            path: rel_path.to_str().unwrap().to_owned(), // todo: make safe
            parent_dir,
            entries,
        })
        .into_response()
}
