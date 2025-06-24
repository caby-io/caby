use crate::{config::Config, ctx::Ctx, error::Result, jsend::JSendBuilder, Error};
use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf, pin::pin, str::FromStr};
use tokio::{
    fs::{self, remove_file, OpenOptions},
    io::{AsyncWriteExt, BufReader, BufWriter},
};
use tokio_util::io::StreamReader;
use tracing::debug;

#[derive(Deserialize, Debug)]
enum UploadEntryType {
    Directory,
    File,
}

// todo: need to consider how best to handle empty dirs
#[derive(Deserialize, Debug)]
pub struct UploadEntry {
    entry_type: UploadEntryType,
    name: String,
    size: Option<u64>,
    xxh_digest: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub enum ConflictStrategy {
    Override,
    Skip,
    Prompt,
    Deconflict,
}

#[derive(Deserialize, Debug)]
pub struct RegisterUploadRequest {
    pub base_path: String,
    pub entries: Vec<UploadEntry>,
    pub conflict_strategy: ConflictStrategy,
}

#[derive(Serialize)]
struct RegisterUploadResponse {
    pub id: String,
    pub chunk_size: u32,
}

// todo: return a signed token or JWT
pub async fn handle_register_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    Json(payload): Json<RegisterUploadRequest>,
) -> Response {
    debug!("{:?}", payload);

    // Validate?
    // Generate an ID for this request
    let id = xid::new();
    // Create a tmp dir for this upload
    fs::create_dir(&cfg.uploads_path.join(id.to_string())).await;

    // Create a tmp file for this upload
    // TODO

    JSendBuilder::new()
        .success(RegisterUploadResponse {
            id: id.to_string(),
            chunk_size: 1 * 1024 * 1024, // todo: tune
        })
        .into_response()
}

// const HEADER_UPLOAD_ID: &str = "Caby-Upload-ID";
// const HEADER_UPLOAD_FILE: &str = "Caby-Upload-File";
const HEADER_UPLOAD_CHUNK: &str = "Caby-Upload-Chunk";

fn get_header_value(headers: &HeaderMap, key: &str) -> Result<String> {
    Ok(headers
        .get(key)
        .ok_or(Error::HeaderMissing(key.into()))?
        .to_str()
        .map_err(|_| Error::Generic("couldn't convert header value into str".into()))?
        .to_owned())
}

pub async fn handle_chunk_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    headers: HeaderMap,
    Path((id, file)): Path<(String, String)>,
    body: Body,
) -> Response {
    let id_path = PathBuf::from(id);
    let file_path = PathBuf::from(file);
    // let upload_chunk: String = match get_header_value(&headers, HEADER_UPLOAD_CHUNK) {
    //     Ok(v) => v,
    //     Err(e) => return e.into_response(),
    // };

    let resp = JSendBuilder::new();

    let full_path = cfg.uploads_path.join(id_path).join(file_path);

    println!("{:?}", full_path.clone());

    // todo: validate
    // todo: get a JWT or other (PASETO?) so we don't have to read the config

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

    // TODO: move to fn
    let body_with_io_error = body
        .into_data_stream()
        .take(10_001) // todo: make this dynamic
        .map_err(io::Error::other);
    let mut body_reader = pin!(StreamReader::new(body_with_io_error));

    // todo: handle error
    let bytes_written = tokio::io::copy(&mut body_reader, &mut file).await.unwrap();
    if bytes_written > 10_000 {
        // todo: handle error
        remove_file(full_path).await.unwrap();

        return resp
            .fail("bytes received exceeded negotiated size")
            .status_code(StatusCode::BAD_REQUEST)
            .into_response();
    }

    resp.success("ok")
        .status_code(StatusCode::CREATED)
        .into_response()
}
