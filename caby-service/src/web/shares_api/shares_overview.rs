use std::path::Path as StdPath;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::warn;

use crate::{
    auth::{authorize_share, AuthUser},
    config::Config,
    files::overview::{build_overview, OverviewEntry},
    jsend::JSendBuilder,
    share::Share,
    user::Permission,
    web::shares_api::ShareIdParam,
};

const SHARE_OVERVIEW_MAX_DEPTH: u32 = 20;

#[derive(Serialize)]
struct ShareOverviewResponse {
    entries: Vec<OverviewEntry>,
}

pub async fn handle_share_overview(
    State(cfg): State<Config>,
    auth: Option<AuthUser>,
    Path(params): Path<ShareIdParam>,
) -> Response {
    let resp = JSendBuilder::new();

    let (space, share) = match Share::resolve(&cfg, &params.id).await {
        Ok(Some(resolved)) => resolved,
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

    if !authorize_share(auth.as_ref(), &share, Permission::View) {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("not authorized for this share")
            .into_response();
    }

    let root = match share.scope_path(&space, StdPath::new("")) {
        Ok(root) => space.live.join(root),
        Err(_) => return resp.fail("invalid share root").into_response(),
    };

    let entries =
        match build_overview(&root, StdPath::new(""), SHARE_OVERVIEW_MAX_DEPTH, true).await {
            Ok(entries) => entries,
            Err(err) => {
                warn!("could not build overview for share {}: {:#}", share.id, err);
                return resp.internal_error().into_response();
            }
        };

    resp.success(ShareOverviewResponse { entries })
        .into_response()
}
