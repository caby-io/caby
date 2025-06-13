use crate::{config::Config, ctx::Ctx, error::Result, files::joined_path, jsend};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

pub async fn handle_download_files(
    State(cfg): State<Config>,
    ctx: Result<Ctx>,
    files_path: Option<Path<String>>,
) -> Response {
    let rel_path = match files_path {
        Some(Path(p)) => PathBuf::from(p),
        None => return (StatusCode::NOT_FOUND, "file path required").into_response(),
    };

    let Some(path) = joined_path(&cfg.live_path, &rel_path) else {
        return jsend::JSendBuilder::new()
            .fail("invalid path")
            .into_response();
    };

    if !path.is_file() {
        return (
            StatusCode::BAD_REQUEST,
            "only files are supported at the moment",
        )
            .into_response();
    }

    let file = match tokio::fs::File::open(path.clone()).await {
        Ok(file) => file,
        Err(err) => {
            return (StatusCode::NOT_FOUND, format!("File not found: {}", err)).into_response()
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
        (header::CONTENT_TYPE, &format!("{:?}", content_type)),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", filename.to_str().unwrap()),
        ),
    ];

    (headers, body).into_response()
}
