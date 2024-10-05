use crate::error::Result;
use crate::files::{get_entries, Directory, File};
use crate::jsend::JSendBuilder;
use crate::{ctx::Ctx, jsend};
use axum::body::{Body, BodyDataStream};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::Path, routing::get, Extension, Json, Router};
use path_clean::{clean, PathClean};
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tokio_util::bytes::BytesMut;
use tokio_util::io::ReaderStream;
use tracing::{debug, error};

pub fn routes() -> Router {
    Router::new()
        .route("/files/", get(api_files))
        .route("/files/*file_path", get(api_files))
        .route("/download/*file_path", get(download_files))
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
    let path = files_path.map_or(PathBuf::from(""), |Path(p)| clean(p));
    let resp = jsend::JSendBuilder::new();

    // todo: get base path from a var
    // todo: consider santizing after join
    // todo: check that it is a dir? OR return something else for files
    let (dirs, files) =
        match get_entries(PathBuf::from("/").join(&path).clean().as_path(), &path).await {
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

async fn download_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    let path = match files_path {
        Some(Path(p)) => PathBuf::from("/").join(p.clone()).clean(),
        None => return (StatusCode::NOT_FOUND, "file path required").into_response(),
    };
    // let resp = jsend::JSendBuilder::new();

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
