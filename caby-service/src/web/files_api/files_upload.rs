use crate::{
    config::Config,
    ctx::Ctx,
    error::Result,
    jsend::JSendBuilder,
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
use serde::{Deserialize, Serialize};
use std::{
    hash::Hasher,
    io::{self},
    os::unix::fs::MetadataExt,
    path::PathBuf,
};
use tokio::{
    fs::{self, remove_file, File, OpenOptions},
    io::{AsyncReadExt, BufReader},
};
use tokio_util::io::StreamReader;
use tracing::error;
use xxhash_rust::xxh64::Xxh64;

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

// todo: return error on empty entries
pub async fn handle_register_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    Json(req): Json<RegisterUploadRequest>,
) -> Response {
    // Validate?
    // Generate an ID for this request
    let id = xid::new();
    // Create an upload dir for this upload
    fs::create_dir(&cfg.uploads_path.join(id.to_string())).await;

    // Create an meta file for this upload
    // TODO

    // todo: make a builder function for this
    let token_payload = UploadTokenPayload {
        id: id.to_string(),
        base_path: req.base_path,
        chunk_size: MAX_CHUNK_SIZE,
        files: req
            .entries
            .into_iter()
            .filter(|e| matches!(e.entry_type, UploadEntryType::File))
            .map(|e| TokenFile {
                name: e.name.clone(),
                size: e.size.clone(),
            })
            .collect(),
    };

    JSendBuilder::new()
        .success(RegisterUploadResponse {
            id: id.to_string(),
            chunk_size: MAX_CHUNK_SIZE,
            token: token_payload.into(),
        })
        .into_response()
}

pub async fn handle_chunk_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    headers: HeaderMap,
    Path((id, file)): Path<(String, String)>,
    body: Body,
) -> Response {
    let resp = JSendBuilder::new();

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload: UploadTokenPayload = match upload_token_str.to_string().try_into() {
        Ok(r) => r,
        Err(e) => return e.into_response(),
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
    if id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // Check that the file was registered in the token
    let Some(token_file) = upload_token_payload.files.iter().find(|f| f.name == file) else {
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

    let id_path = PathBuf::from(id);
    let file_path = PathBuf::from(file);

    let full_path = cfg.uploads_path.join(id_path).join(file_path);

    let mut file = match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(full_path.clone())
        .await
    {
        Ok(f) => f,
        // todo: log the error
        Err(e) => return resp.error("couldn't open file for writing").into_response(),
    };

    // todo: move to fn
    let mut limited_body_stream =
        StreamReader::new(body.into_data_stream().map_err(io::Error::other))
            .take((upload_token_payload.chunk_size + 1).into());

    // todo: handle error
    let bytes_written = tokio::io::copy(&mut limited_body_stream, &mut file)
        .await
        .expect("couldn't copy bytes");

    if bytes_written > upload_token_payload.chunk_size.into() {
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

// todo: actually store this data somewhere
// this handler is for updating file metadata such as the hash or whether the file is complete
pub async fn handle_update_file(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    headers: HeaderMap,
    Path((id, file)): Path<(String, String)>,
    Json(body): Json<UpdateFileRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload: UploadTokenPayload = match upload_token_str.to_string().try_into() {
        Ok(r) => r,
        Err(err) => return err.into_response(),
    };

    let id_path = PathBuf::from(id.clone());
    let file_path = PathBuf::from(file.clone());

    if id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // Check that the file was registered in the token
    let Some(token_file) = upload_token_payload
        .files
        .into_iter()
        .find(|f| f.name == file)
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

    let full_path = cfg.uploads_path.join(id_path).join(file_path);
    let (disk_digest, disk_size) = match get_file_digest_size(full_path).await {
        Ok(d) => d,
        Err(err) => return err.into_response(),
    };

    if disk_digest != body_digest {
        println!("{}", disk_digest);
        return resp.fail("digest mismatch").into_response();
    }
    if disk_size != body_size {
        return resp.fail("size mismatch").into_response();
    }

    // note: this doesn't actually do anything
    // this should, eventually, compile the chunk files to indicate that the file is ready
    resp.success("file marked as completed")
        .status_code(StatusCode::OK)
        .into_response()
}

pub async fn handle_complete_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    headers: HeaderMap,
    Path((id)): Path<(String)>,
) -> Response {
    let resp = JSendBuilder::new();

    // parse the upload token
    let upload_token_str = match get_required_header(&headers, HEADER_UPLOAD_TOKEN) {
        Ok(v) => v,
        Err(err) => return err.into_response(),
    };

    let upload_token_payload: UploadTokenPayload = match upload_token_str.to_string().try_into() {
        Ok(r) => r,
        Err(e) => return e.into_response(),
    };

    let id_path = PathBuf::from(id.clone());

    if id != upload_token_payload.id {
        return resp
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("token/request upload id mismatch")
            .into_response();
    }

    // todo: check that all the files are complete
    // todo: get the base path
    let upload_path = cfg.uploads_path.join(id_path);
    let mut entries = match fs::read_dir(upload_path).await {
        Ok(e) => e,
        // todo: improve this error
        Err(err) => return resp.internal_error().into_response(),
    };

    // todo: improve err
    while let Some(dir_entry) = entries.next_entry().await.expect("couldn't get next entry") {
        // let filename = dir_entry.file_name();
        let dest = cfg.live_path.join(dir_entry.file_name());
        fs::rename(dir_entry.path(), dest)
            .await
            .expect("couldn't move file");
    }

    resp.status_code(StatusCode::CREATED)
        .success("created")
        .into_response()
}
