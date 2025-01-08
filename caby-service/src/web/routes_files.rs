use crate::{
    ctx::Ctx,
    error::Result,
    files::{build_entries, Entry},
    jsend,
    jsend::JSendBuilder,
};
use axum::{
    body::{Body, BodyDataStream},
    extract,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get},
    Extension, Json, Router,
};
use path_clean::{clean, PathClean};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::fs;
use tokio_util::{bytes::BytesMut, io::ReaderStream};
use tracing::{debug, error};

pub fn routes() -> Router {
    Router::new()
        .route("/files/", get(api_files))
        .route("/files/*file_path", get(api_files))
        .route("/download/*file_path", get(download_files))
        .route("/files", delete(delete_entries))
}

#[derive(Serialize)]
struct FilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub entries: Vec<Entry>,
}

async fn api_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    // todo: sanitize path, more
    let path = files_path.map_or(PathBuf::from(""), |Path(p)| clean(p));
    let resp = jsend::JSendBuilder::new();

    // todo: get base path from a var
    // todo: consider santizing after join
    // todo: check that it is a dir? OR return something else for files
    let entries = match build_entries(PathBuf::from("/").join(&path).clean().as_path(), &path).await
    {
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
            entries,
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

#[derive(Deserialize)]
struct DeleteEntriesRequest {
    pub entries: Vec<String>,
    pub force: bool,
}

#[derive(Serialize)]
struct DeleteEntriesResponse {
    pub deleted: Vec<String>,
    pub errors: Vec<String>,
}

// todo: this should be archiving instead of deleting
async fn delete_entries(
    ctx: Result<Ctx>,
    extract::Json(req): extract::Json<DeleteEntriesRequest>,
) -> Response {
    // todo: validate that they're valid paths

    let mut deleted = vec![];
    let mut errors = vec![];

    for relative_path in req.entries {
        let path = PathBuf::from("/").join(relative_path).clean();

        let Ok(metadata) = fs::metadata(path.clone()).await else {
            // todo: make error structured and parseable
            errors.push(format!("{:?} not found", path));
            debug!("couldn't get entry at {:?}", path);
            continue;
        };

        if metadata.is_dir() {
            debug!("got a directory at {:?}", path);
            // fs::remove_dir_all(path);
            deleted.push(path.as_os_str().to_str().unwrap().to_owned());
            continue;
        }
        debug!("got a file at {:?}", path);
        // fs::remove_file(path);
        deleted.push(path.as_os_str().to_str().unwrap().to_owned());
    }

    jsend::JSendBuilder::new()
        .success(DeleteEntriesResponse {
            deleted, // todo: make safe
            errors,
        })
        .into_response()
}
