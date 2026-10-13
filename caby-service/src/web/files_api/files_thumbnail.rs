use std::path::PathBuf;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use jiff::Timestamp;
use serde::Deserialize;
use tokio_util::io::ReaderStream;
use tracing::{debug, warn};

use crate::{
    config::Config,
    files::{
        media,
        thumbnail::{self, ThumbError},
    },
    jsend::JSendBuilder,
    space::{Space, SpaceDir},
    web::files_api::files_list::FilesPathParams,
};

#[derive(Deserialize)]
pub struct ThumbTokenQuery {
    token: Option<String>,
}

pub async fn handle_get_thumbnail(
    State(cfg): State<Config>,
    space: Space,
    Query(query): Query<ThumbTokenQuery>,
    path_params: Path<FilesPathParams>,
) -> Response {
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);

    // authorize against the directory-scoped capability token in the query string
    let unauthorized_resp = || {
        JSendBuilder::new()
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("unauthorized")
            .into_response()
    };

    let Some(token) = query.token else {
        return unauthorized_resp();
    };
    let expires_at = match media::try_verify_file(&cfg, &space.name, &rel_path, &token) {
        Ok(expires_at) => expires_at,
        Err(err) => {
            debug!("could not verify media token: {:#}", err);
            return unauthorized_resp();
        }
    };

    let Ok(live_path) = space.join(SpaceDir::LIVE, &rel_path) else {
        return JSendBuilder::new().fail("invalid path").into_response();
    };

    let Ok(thumb_path) = thumbnail::thumb_path(&space, &rel_path) else {
        return JSendBuilder::new().fail("invalid path").into_response();
    };

    if !tokio::fs::try_exists(&thumb_path).await.unwrap_or(false) {
        if !live_path.is_file() {
            return JSendBuilder::new()
                .status_code(StatusCode::NOT_FOUND)
                .fail("file not found")
                .into_response();
        }

        match thumbnail::try_generate_thumb(&live_path, &thumb_path, cfg.img_thumbs.max_edge).await
        {
            Ok(()) => {}
            Err(ThumbError::UnsupportedFormat) => {
                return JSendBuilder::new()
                    .status_code(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .fail("unsupported image format")
                    .into_response();
            }
            Err(err) => {
                warn!("could not generate thumbnail for {:?}: {}", rel_path, err);
                return JSendBuilder::new().internal_error().into_response();
            }
        }
    }

    let file = match tokio::fs::File::open(&thumb_path).await {
        Ok(f) => f,
        Err(err) => {
            warn!("could not open thumbnail {:?}: {:#}", thumb_path, err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let max_age = (expires_at - Timestamp::now().as_second()).max(0);
    let cache_control = format!("private, max-age={max_age}, immutable");
    let headers = [
        (header::CONTENT_TYPE, "image/webp"),
        (header::CACHE_CONTROL, cache_control.as_str()),
    ];

    (headers, body).into_response()
}
