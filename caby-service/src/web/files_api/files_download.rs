use crate::{
    auth::AuthorizedUser,
    config::Config,
    download,
    jsend::JSendBuilder,
    space::{Space, SpaceDir},
    web::{extractors::DownloadUser, files_api::files_list::FilesPathParams},
};
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Json, Path, State},
    http::header,
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio_util::io::ReaderStream;
use tracing::warn;

#[derive(Deserialize)]
pub struct RegisterDownloadRequest {
    pub files: Vec<String>,
}

#[derive(Serialize)]
struct RegisterDownloadResponse {
    pub token: download::Token,
}

pub async fn handle_register_download(
    State(cfg): State<Config>,
    space: Space,
    user: AuthorizedUser,
    Json(req): Json<RegisterDownloadRequest>,
) -> Response {
    let mut cleaned_files = Vec::with_capacity(req.files.len());
    for file in &req.files {
        let rel_path = PathBuf::from(file).clean();
        if space.join(SpaceDir::LIVE, &rel_path).is_err() {
            return JSendBuilder::new()
                .fail(format!("invalid path: {}", file))
                .into_response();
        }
        cleaned_files.push(rel_path.to_string_lossy().into_owned());
    }

    let token = match download::Token::new(&space.name, cleaned_files) {
        Ok(t) => t,
        Err(err) => {
            warn!("could not generate download token: {:#}", err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let download_file = user.user.path.join(format!("download_{}", token.value));
    if let Err(err) = fs::write(&download_file, token.to_file_string()).await {
        warn!(
            "could not write download file for user {}: {:#}",
            user.user.name, err
        );
        return JSendBuilder::new().internal_error().into_response();
    }

    JSendBuilder::new()
        .success(RegisterDownloadResponse { token })
        .into_response()
}

pub async fn handle_download_files(
    State(cfg): State<Config>,
    space: Space,
    user: DownloadUser,
    path_params: Path<FilesPathParams>,
) -> Response {
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);

    let Ok(path) = space.join(SpaceDir::LIVE, &rel_path) else {
        return JSendBuilder::new().fail("invalid path").into_response();
    };

    if !path.is_file() {
        return JSendBuilder::new()
            .fail("only files supported")
            .into_response();
    }

    let file = match tokio::fs::File::open(path.clone()).await {
        Ok(file) => file,
        Err(err) => {
            return JSendBuilder::new()
                .fail(format!("file not found: {}", err))
                .into_response();
        }
    };

    let filename = path.file_name().unwrap();
    // todo: make this mime type force a download
    // todo: make mime or download be based on argument
    let content_type = mime_guess::from_path(&path)
        .first_raw()
        .unwrap_or("application/octet-stream");

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, &content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename.to_str().unwrap()),
        ),
    ];

    (headers, body).into_response()
}
