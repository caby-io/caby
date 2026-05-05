use crate::{
    auth::AuthorizedUser,
    config::Config,
    error::RequestError,
    files::merge_dir,
    jsend::JSendBuilder,
    space::Space,
    upload::{
        decode_upload_token, generate_upload_token, get_file_digest_size, UploadToken,
        UploadTokenPayload,
    },
    web::{headers::get_required_header, upload::*},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::TryStreamExt;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::{
    hash::Hasher,
    io::{self},
    os::unix::fs::MetadataExt,
    path::{Component, PathBuf},
};
use tokio::{
    fs::{self, remove_file, OpenOptions},
    io::AsyncReadExt,
};
use tokio_util::io::StreamReader;
use tracing::error;

#[derive(Deserialize, Debug)]
pub struct RegisterUploadRequest {
    pub base_path: String,
    pub entries: Vec<UploadEntry>,
    pub conflict_strategy: ConflictStrategy,
}

#[derive(Serialize)]
struct RegisterUploadResponse {
    pub id: String,
    pub chunk_size: u64,
    pub token: UploadToken,
}

// rejects absolute paths and any path that escapes its root via `..` after cleaning
fn is_safe_relative_path(p: &str) -> bool {
    let cleaned = PathBuf::from(p).clean();
    if cleaned.is_absolute() {
        return false;
    }
    !matches!(cleaned.components().next(), Some(Component::ParentDir))
}

// todo: return error on empty entries
pub async fn handle_register_upload(
    cfg: State<Config>,
    space: Space,
    user: AuthorizedUser,
    Json(req): Json<RegisterUploadRequest>,
) -> Response {
    // basic validation
    // todo: improve
    if !is_safe_relative_path(&req.base_path) {
        return JSendBuilder::new()
            .fail(format!("invalid base_path: {}", req.base_path))
            .into_response();
    }
    for entry in &req.entries {
        if !is_safe_relative_path(&entry.name) {
            return JSendBuilder::new()
                .fail(format!("invalid entry name: {}", entry.name))
                .into_response();
        }
    }

    // Generate an ID for this request
    let id = xid::new();
    // Create an upload dir for this upload
    fs::create_dir(&space.uploads().join(id.to_string())).await;

    // Create an meta file for this upload
    // TODO

    // todo: make a builder function for this
    let token_payload = UploadTokenPayload {
        id: id.to_string(),
        base_path: req.base_path,
        chunk_size: MAX_CHUNK_SIZE,
        // files: req
        //     .entries
        //     .into_iter()
        //     .filter(|e| matches!(e.entry_type, UploadEntryType::File))
        //     .map(|e| TokenFile {
        //         name: e.name.clone(),
        //         size: e.size,
        //     })
        //     .collect(),
    };

    let token = match generate_upload_token(&cfg, token_payload) {
        Ok(t) => t,
        Err(err) => {
            error!("could not generate upload token: {:#}", err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    JSendBuilder::new()
        .success(RegisterUploadResponse {
            id: id.to_string(),
            chunk_size: MAX_CHUNK_SIZE,
            token,
        })
        .into_response()
}

#[derive(Deserialize)]
pub struct UploadChunkParams {
    pub id: String,
    pub file_path: String,
}

pub async fn handle_upload_chunk(
    cfg: State<Config>,
    space: Space,
    authorized_user: AuthorizedUser,
    headers: HeaderMap,
    path_params: Path<UploadChunkParams>,
    body: Body,
) -> Response {
    let resp = JSendBuilder::new();

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload = match decode_upload_token(&cfg, &upload_token_str) {
        Ok(p) => p,
        Err(err) => {
            error!("could not decode upload token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    // note: this should enable async chunk upload eventually
    // determine the chunk index
    let chunk_index_str = match get_required_header(&headers, HEADER_CHUNK_INDEX) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };
    let Ok(chunk_index) = chunk_index_str.parse::<u64>() else {
        return resp.fail("chunk index must be a number").into_response();
    };

    // todo: check optional hash header for this chunk

    // todo: check content length header against the chunk size

    // check that the file is registered to this token
    if path_params.id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // Check that the file was registered in the token
    let Some(token_file) = upload_token_payload
        .files
        .iter()
        .find(|f| f.name == path_params.file_path)
    else {
        return resp
            .fail("requested file is not a part of this upload token")
            .into_response();
    };

    // check that chunk index is in range
    let max_chunks = (token_file.size.expect("missing token file size") as f64
        / upload_token_payload.chunk_size as f64)
        .ceil() as u64;
    if chunk_index > max_chunks {
        return resp.fail("chunk index out of range").into_response();
    }
    // todo: make this more concise
    // check that the chunk index is valid
    // if (upload_token_payload.chunk_size as f64 / token_file.size.unwrap() as f64).floor()
    //     <= chunk_index as f64
    // {
    //     println!("FAILED HERE B");
    //     println!(
    //         (upload_token_payload.chunk_size as f64 / token_file.size.unwrap() as f64).floor()
    //     );
    //     return resp.fail("chunk index out of bounds").into_response();
    // }

    // we have validated the upload, start processing the chunks

    // let id_path = PathBuf::from(id);
    // let file_path = PathBuf::from(file);

    let full_path = space
        .uploads()
        .join(&path_params.id)
        .join(&path_params.file_path);

    // ensure the parent dir exists
    let parent_path = match full_path.parent() {
        Some(p) => p,
        None => {
            return resp
                .error("could not determine parent path")
                .into_response()
        }
    };
    if let Err(err) = fs::create_dir_all(parent_path).await {
        error!("could not create parent path for upload: {:#}", err);
        return resp.error("could not create parent path").into_response();
    };

    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&full_path)
        .await
    {
        Ok(f) => f,
        // todo: log the error
        Err(e) => return resp.error("couldn't open file for writing").into_response(),
    };

    // todo: move to fn
    let mut limited_body_stream =
        StreamReader::new(body.into_data_stream().map_err(io::Error::other))
            .take((upload_token_payload.chunk_size + 1));

    // todo: handle error
    let bytes_written = match tokio::io::copy(&mut limited_body_stream, &mut file).await {
        Ok(b) => b,
        Err(err) => {
            error!("could not write chunk: {:#}", err);
            return resp.error("could not write chunk").into_response();
        }
    };

    if bytes_written > upload_token_payload.chunk_size {
        // todo: handle error
        // note: this resets the process until we save individual chunks
        remove_file(full_path)
            .await
            .expect("could not delete oversized file");

        return resp
            .status_code(StatusCode::PAYLOAD_TOO_LARGE)
            .fail("bytes received exceeded negotiated size")
            .into_response();
    }

    resp.success("ok")
        .status_code(StatusCode::CREATED)
        .into_response()
}

#[derive(Deserialize)]
pub struct UpdateFileRequest {
    size: Option<u64>,
    is_complete: Option<bool>,
    xxh_digest: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUploadParams {
    pub id: String,
    pub file_path: Option<String>,
}

// todo: actually store this data somewhere
// this handler is for updating file metadata such as the hash or whether the file is complete
pub async fn handle_update_upload(
    cfg: State<Config>,
    space: Space,
    user: AuthorizedUser,
    headers: HeaderMap,
    path_params: Path<UpdateUploadParams>,
    Json(body): Json<UpdateFileRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload = match decode_upload_token(&cfg, &upload_token_str) {
        Ok(p) => p,
        Err(err) => {
            error!("could not decode upload token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    // let id_path = PathBuf::from(path_params.id.clone());
    // let file_path = PathBuf::from(path_params.file_path.unwrap().clone());

    if path_params.id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // Check that the file was registered in the token
    let Some(token_file) = upload_token_payload
        .files
        .into_iter()
        .find(|f| PathBuf::from(f.name.clone()) == rel_path)
    else {
        return resp
            .fail("requested file is not a part of this upload token")
            .into_response();
    };

    // for now we require all three values in every request to this endpoint
    let Some(body_size) = body.size else {
        return resp.fail("missing file size").into_response();
    };
    let Some(body_is_complete) = body.is_complete else {
        return resp.fail("missing completion status").into_response();
    };
    let Some(body_digest) = body.xxh_digest else {
        return resp.fail("missing xxh digest").into_response();
    };

    if !body_is_complete {
        return resp
            .status_code(StatusCode::UNPROCESSABLE_ENTITY)
            .fail("is_complete required to be true")
            .into_response();
    }

    let full_path = space.uploads().join(&path_params.id).join(&rel_path);
    let (disk_digest, disk_size) = match get_file_digest_size(full_path).await {
        Ok(d) => d,
        Err(err) => return RequestError::from(err).into_response(),
    };

    if disk_digest != body_digest {
        println!("{}", disk_digest);
        return resp.fail("digest mismatch").into_response();
    }
    if disk_size != body_size {
        return resp.fail("size mismatch").into_response();
    }

    // note: this doesn't actually do anything
    // this should, eventually, compile the chunk files to indicate that the upload is complete
    resp.success("file marked as completed")
        .status_code(StatusCode::OK)
        .into_response()
}

#[derive(Deserialize)]
pub struct PublishUploadParams {
    pub id: String,
}

pub async fn handle_publish_upload(
    cfg: State<Config>,
    space: Space,
    user: AuthorizedUser,
    headers: HeaderMap,
    path_params: Path<PublishUploadParams>,
) -> Response {
    let resp = JSendBuilder::new();

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload = match decode_upload_token(&cfg, &upload_token_str) {
        Ok(p) => p,
        Err(err) => {
            error!("could not decode upload token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    // let id_path = PathBuf::from(path_params.id);

    if path_params.id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // todo: check that all the files are complete
    let live_base = space.live().join(&upload_token_payload.base_path);
    let upload_path = space.uploads().join(&path_params.id);
    if let Err(err) = merge_dir(&upload_path, &live_base).await {
        error!("could not publish upload: {:#}", err);
        return resp.internal_error().into_response();
    }

    resp.status_code(StatusCode::CREATED)
        .success("created")
        .into_response()
}
