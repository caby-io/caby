use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;
use tracing::warn;

use crate::{
    config::Config,
    jsend::JSendBuilder,
    share::{download_token, is_filtered, Share},
    space::{Space, SpaceDir},
};

#[derive(Deserialize)]
pub struct DownloadParams {
    id: String,
    path: String,
}

#[derive(Deserialize)]
pub struct DownloadTokenQuery {
    token: Option<String>,
}

pub async fn handle_download_share(
    State(cfg): State<Config>,
    space: Space,
    Path(params): Path<DownloadParams>,
    Query(query): Query<DownloadTokenQuery>,
) -> Response {
    let resp = JSendBuilder::new();

    let unauthorized = || {
        JSendBuilder::new()
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("not authorized for this share")
            .into_response()
    };

    let Some(token) = query.token else {
        return unauthorized();
    };
    let payload = match download_token::decode_token(&cfg, &token) {
        Ok(payload) => payload,
        Err(err) => {
            warn!("could not decode share download token: {:#}", err);
            return unauthorized();
        }
    };
    if payload.is_expired() || payload.share_id != params.id {
        return unauthorized();
    }

    let share = match Share::load(&space, &params.id).await {
        Ok(Some(share)) => share,
        Ok(None) => {
            return resp
                .status_code(StatusCode::NOT_FOUND)
                .fail("share not found")
                .into_response()
        }
        Err(err) => {
            warn!("could not load share {}: {:#}", params.id, err);
            return resp.internal_error().into_response();
        }
    };

    if share.is_expired() {
        return resp
            .status_code(StatusCode::GONE)
            .fail("share has expired")
            .into_response();
    }

    let rel = PathBuf::from(&params.path);
    let scoped = match share.scope_path(&space, &rel) {
        Ok(scoped) => scoped,
        Err(_) => return resp.fail("invalid path").into_response(),
    };

    if scoped.to_string_lossy() != payload.path {
        return unauthorized();
    }

    if scoped
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(is_filtered)
    {
        return resp
            .status_code(StatusCode::NOT_FOUND)
            .fail("file not found")
            .into_response();
    }

    let live = match space.join(SpaceDir::LIVE, &scoped) {
        Ok(live) => live,
        Err(_) => return resp.fail("invalid path").into_response(),
    };

    if !live.is_file() {
        return resp.fail("only files supported").into_response();
    }

    let file = match tokio::fs::File::open(&live).await {
        Ok(file) => file,
        Err(err) => {
            return resp
                .fail(format!("file not found: {}", err))
                .into_response()
        }
    };

    let filename = live
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_owned());
    let content_type = mime_guess::from_path(&live)
        .first_raw()
        .unwrap_or("application/octet-stream");

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = [
        (header::CONTENT_TYPE, content_type.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        ),
    ];

    (headers, body).into_response()
}
