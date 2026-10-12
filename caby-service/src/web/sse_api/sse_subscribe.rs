use std::{
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    controller::SseController,
    jsend::JSendBuilder,
    space::SpaceDir,
    sse::{Subscription, SubscriptionKind},
    web::extractors::RequireAccount,
};

#[derive(Deserialize)]
pub struct SseSubscriptionPathParams {
    connection_id: String,
    kind: SubscriptionKind,
}

#[derive(Deserialize)]
pub struct SseSubscribeRequest {
    space: String,
    path: Option<String>,
}

#[derive(Serialize)]
struct SseSubscribeResponse {
    revision: u64,
}

fn connection_not_found() -> Response {
    JSendBuilder::new()
        .status_code(StatusCode::NOT_FOUND)
        .fail("sse connection not found")
        .into_response()
}

pub async fn handle_put_sse_subscription(
    State(cfg): State<Config>,
    State(sse): State<Arc<SseController>>,
    RequireAccount(account): RequireAccount,
    Path(params): Path<SseSubscriptionPathParams>,
    Json(req): Json<SseSubscribeRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let Some(space) = cfg.find_space(&req.space) else {
        return resp
            .status_code(StatusCode::NOT_FOUND)
            .fail("space not found")
            .into_response();
    };

    let sub = match params.kind {
        SubscriptionKind::Dir => {
            let path = match PathBuf::from(req.path.unwrap_or_default()).clean() {
                path if path == FsPath::new(".") => PathBuf::new(),
                path => path,
            };
            if space.join(SpaceDir::LIVE, &path).is_err() {
                return resp
                    .status_code(StatusCode::BAD_REQUEST)
                    .fail("invalid path")
                    .into_response();
            }
            Subscription::Dir {
                space: space.name,
                path,
            }
        }
        SubscriptionKind::Tree => Subscription::Tree { space: space.name },
        SubscriptionKind::Shares => Subscription::Shares { space: space.name },
    };

    match sse.subscribe(&params.connection_id, &account.name, sub) {
        Some(revision) => resp
            .success(SseSubscribeResponse { revision })
            .into_response(),
        None => connection_not_found(),
    }
}

pub async fn handle_delete_sse_subscription(
    State(sse): State<Arc<SseController>>,
    RequireAccount(account): RequireAccount,
    Path(params): Path<SseSubscriptionPathParams>,
) -> Response {
    match sse.unsubscribe(&params.connection_id, &account.name, params.kind) {
        true => JSendBuilder::new().success("unsubscribed").into_response(),
        false => connection_not_found(),
    }
}
