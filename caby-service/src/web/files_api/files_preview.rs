use std::path::PathBuf;

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tracing::debug;

use crate::{
    config::Config,
    files::media,
    jsend::JSendBuilder,
    space::{Space, SpaceDir},
    web::files_api::files_list::FilesPathParams,
};

#[derive(Deserialize)]
pub struct PreviewTokenQuery {
    token: Option<String>,
}

// Handles streaming of media
pub async fn handle_get_preview(
    State(cfg): State<Config>,
    space: Space,
    Query(query): Query<PreviewTokenQuery>,
    path_params: Path<FilesPathParams>,
    request: Request,
) -> Response {
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);

    let unauthorized_resp = || {
        JSendBuilder::new()
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("unauthorized")
            .into_response()
    };

    let Some(token) = query.token else {
        return unauthorized_resp();
    };
    if let Err(err) = media::try_verify_file(&cfg, &space.name, &rel_path, &token) {
        debug!("could not verify media token: {:#}", err);
        return unauthorized_resp();
    }

    let Ok(live_path) = space.join(SpaceDir::LIVE, &rel_path) else {
        return JSendBuilder::new().fail("invalid path").into_response();
    };

    if !live_path.is_file() {
        return JSendBuilder::new()
            .status_code(StatusCode::NOT_FOUND)
            .fail("file not found")
            .into_response();
    }

    ServeFile::new(live_path)
        .oneshot(request)
        .await
        .unwrap()
        .into_response()
}
