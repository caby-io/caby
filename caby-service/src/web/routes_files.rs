use crate::error::Result;
use crate::files::{get_entries, Directory, File};
use crate::jsend::JSendBuilder;
use crate::{ctx::Ctx, jsend};
use axum::response::{IntoResponse, Response};
use axum::{extract::Path, routing::get, Extension, Json, Router};
use path_clean::{clean, PathClean};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tracing::{debug, error};

pub fn routes() -> Router {
    Router::new()
        .route("/files/", get(api_files))
        .route("/files/*file_path", get(api_files))
}

#[derive(Serialize)]
struct FilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub dirs: Vec<Directory>,
    pub files: Vec<File>,
}

async fn api_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    // todo: sanitize path, more
    let path = files_path.map_or(PathBuf::from("/"), |Path(p)| clean(p));
    let resp = jsend::JSendBuilder::new();

    // todo: get base path from a var
    // todo: consider santizing after join
    let Ok((dirs, files)) = get_entries(PathBuf::from("/").join(&path).clean().as_path()).await
    else {
        return resp.error("could not access files").into_response();
    };

    // todo: make safe
    let parent_dir = PathBuf::from(&path).parent().map(|p| {
        debug!("{:?}", p);
        p.to_str().unwrap().to_owned()
    });

    jsend::JSendBuilder::new()
        .success(FilesResponse {
            path: path.to_str().unwrap().to_owned(), // todo: make safe
            parent_dir,
            dirs,
            files,
        })
        .into_response()
}
