use crate::{
    config::Config,
    ctx::Ctx,
    error::Result,
    files::{
        upload::{TokenFile, UploadToken, UploadTokenPayload},
        EntryType,
    },
    jsend::JSendBuilder,
    Error,
};
use axum::{
    body::{to_bytes, Body},
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::{
    io::{self},
    path::PathBuf,
    pin::pin,
    str::FromStr,
};
use tokio::{
    fs::{self, remove_file, OpenOptions},
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader},
};
use tokio_util::io::StreamReader;
use tracing::debug;

static MAX_CHUNK_SIZE: u32 = 10_000_000;

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
    pub token: UploadToken,
}

// todo: return a signed token or JWT
pub async fn handle_register_upload(
    cfg: State<Config>,
    ctx: Result<Ctx>,
    Json(body): Json<RegisterUploadRequest>,
) -> Response {
    debug!("{:?}", body);

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
        chunk_size: MAX_CHUNK_SIZE,
        files: body
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

// const HEADER_UPLOAD_ID: &str = "Caby-Upload-ID";
// const HEADER_UPLOAD_FILE: &str = "Caby-Upload-File";
const HEADER_UPLOAD_TOKEN: &str = "Caby-Upload-Token";
const HEADER_UPLOAD_CHUNK: &str = "Caby-Upload-Chunk";

fn get_header_value(headers: &HeaderMap, key: &str) -> Result<String> {
    Ok(headers
        .get(key)
        .ok_or(Error::HeaderMissing(key.into()))?
        .to_str()
        .map_err(|_| Error::Generic("couldn't convert header value into str".into()))?
        .to_owned())
}

fn get_upload_token_header(headers: &HeaderMap) -> Result<UploadTokenPayload> {
    let header_value = get_header_value(headers, HEADER_UPLOAD_TOKEN)?;
    Ok(header_value.try_into()?)
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

    let token_payload = match get_upload_token_header(&headers) {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    println!("{:?}", token_payload);
    // let upload_chunk: String = match get_header_value(&headers, HEADER_UPLOAD_CHUNK) {
    //     Ok(v) => v,
    //     Err(e) => return e.into_response(),
    // };

    let resp = JSendBuilder::new();

    let full_path = cfg.uploads_path.join(id_path).join(file_path);

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
    let mut limited_body_stream =
        StreamReader::new(body.into_data_stream().map_err(io::Error::other)).take(10_000_001);

    // todo: handle error
    let bytes_written = tokio::io::copy(&mut limited_body_stream, &mut file)
        .await
        .expect("couldn't copy bytes");

    // body_reader.poll_fill_buf(cx, buf)

    println!("{:?}", bytes_written);

    if bytes_written > 10_000_000 {
        // todo: handle error
        // remove_file(full_path)
        //     .await
        //     .expect("could not delete oversized file");

        return resp
            .fail("bytes received exceeded negotiated size")
            .status_code(StatusCode::BAD_REQUEST)
            .into_response();
    }

    resp.success("ok")
        .status_code(StatusCode::CREATED)
        .into_response()
}
