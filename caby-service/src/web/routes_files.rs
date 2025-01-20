use crate::{
    ctx::Ctx,
    error::Result,
    files::{build_entries, joined_path, Entry},
    jsend::{self, JSendBuilder},
};
use axum::{
    body::{Body, BodyDataStream},
    extract::{self, Path},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
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
        .route("/files/rename", post(rename_entries))
}

static ROOT_PATH: &str = "/home/suhaib/caby-home";

#[derive(Serialize)]
struct FilesResponse {
    pub path: String,
    pub parent_dir: Option<String>,
    pub entries: Vec<Entry>,
}

async fn api_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    // todo: sanitize path, more
    let root_path = PathBuf::from(ROOT_PATH);
    let rel_path = files_path.map_or(PathBuf::from(""), |Path(p)| PathBuf::from(p));

    let Some(path) = joined_path(&root_path, &rel_path) else {
        return jsend::JSendBuilder::new()
            .fail("invalid path")
            .into_response();
    };

    let resp = jsend::JSendBuilder::new();

    // todo: get base path from a var
    // todo: consider santizing after join
    // todo: check that it is a dir? OR return something else for files
    let entries = match build_entries(&root_path, &path).await {
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
        .success(FilesResponse {
            path: rel_path.to_str().unwrap().to_owned(), // todo: make safe
            parent_dir,
            entries,
        })
        .into_response()
}

async fn download_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    let rel_path = match files_path {
        Some(Path(p)) => PathBuf::from(p),
        None => return (StatusCode::NOT_FOUND, "file path required").into_response(),
    };

    let root_path = PathBuf::from(ROOT_PATH);
    let Some(path) = joined_path(&root_path, &rel_path) else {
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
    let root_path = PathBuf::from(ROOT_PATH);

    let mut deleted = vec![];
    let mut errors = vec![];

    for relative_path in req.entries {
        let rel_path = PathBuf::from(relative_path.clone()).clean();
        let Some(path) = joined_path(&root_path, &rel_path) else {
            // todo
            errors.push(format!("{:?} invaild path", relative_path));
            continue;
        };

        let Ok(metadata) = fs::metadata(path.clone()).await else {
            // todo: make error structured and parseable
            errors.push(format!("{:?} not found", relative_path));
            continue;
        };

        if metadata.is_dir() {
            if let Err(err) = fs::remove_dir_all(path).await {
                errors.push(format!("couldn't delete {:?}: {:?}", relative_path, err));
                continue;
            }
            deleted.push(rel_path.to_str().unwrap().to_owned());
            continue;
        }

        if let Err(err) = fs::remove_file(path).await {
            errors.push(format!("couldn't delete {:?}: {:?}", relative_path, err));
            continue;
        }
        deleted.push(rel_path.to_str().unwrap().to_owned());
    }

    jsend::JSendBuilder::new()
        .success(DeleteEntriesResponse { deleted, errors })
        .into_response()
}

#[derive(Deserialize)]
struct RenamedEntriesRequest {
    pub entries: Vec<(String, String)>,
    pub force: bool,
}

#[derive(Serialize)]
struct RenamedEntriesResponse {
    pub renamed: Vec<(String, String)>,
    pub errors: Vec<String>,
}

async fn rename_entries(
    ctx: Result<Ctx>,
    extract::Json(req): extract::Json<RenamedEntriesRequest>,
) -> Response {
    let root_path = PathBuf::from(ROOT_PATH);

    let mut renamed = vec![];
    let mut errors = vec![];

    for (input_src, input_dst) in req.entries {
        // Build & validate source path
        let src_rpath = PathBuf::from(input_src.clone()).clean();
        let Some(src_path) = joined_path(&root_path, &src_rpath) else {
            // todo: make error structured and parseable
            errors.push(format!("{:?} invaild source path", input_src));
            continue;
        };

        let Ok(src_metadata) = fs::metadata(src_path.clone()).await else {
            // todo: make error structured and parseable
            errors.push(format!("source {:?} not found", src_rpath));
            continue;
        };

        // Build & validate destination path
        let dst_rpath = PathBuf::from(input_dst.clone()).clean();
        let Some(dst_path) = joined_path(&root_path, &dst_rpath) else {
            // todo: make error structured and parseable
            errors.push(format!("{:?} invaild destination path", input_src));
            continue;
        };

        let Ok(exists) = fs::try_exists(dst_path.clone()).await else {
            // todo: choose whether to display or output the error
            errors.push(format!(
                "{:?}: couldn't determine whether destination path exists",
                dst_path
            ));
            continue;
        };

        if exists {
            errors.push(format!("{:?}: destination exists", dst_path));
            continue;
        }

        if let Err(err) = fs::rename(src_path, dst_path).await {
            errors.push(format!(
                "couldn't rename {:?} to {:?}: {:?}",
                src_rpath, dst_rpath, err
            ));
            continue;
        }

        renamed.push((
            src_rpath.to_str().unwrap().to_owned(),
            dst_rpath.to_str().unwrap().to_owned(),
        ));
    }

    jsend::JSendBuilder::new()
        .success(RenamedEntriesResponse { renamed, errors })
        .into_response()
}
