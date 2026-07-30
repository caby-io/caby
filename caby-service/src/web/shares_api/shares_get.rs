use std::path::Path as StdPath;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    auth::AuthUser,
    jsend::JSendBuilder,
    share::{Share, ShareAuth},
    space::Space,
};

#[derive(Deserialize)]
pub struct ShareIdParam {
    pub id: String,
}

#[derive(Serialize)]
struct AuthOptions {
    open: bool,
    password: bool,
}

#[derive(Serialize)]
struct GetShareResponse {
    id: String,
    space: String,
    root_name: String,
    auth: AuthOptions,
}

pub async fn handle_get_share(
    space: Space,
    auth: Option<AuthUser>,
    Path(params): Path<ShareIdParam>,
) -> Response {
    let resp = JSendBuilder::new();

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

    // We only care about the minimum info for what UI to present to the user
    let mut auth_options = AuthOptions {
        open: false,
        password: false,
    };

    for flow in share.guest_flows.iter() {
        match flow.auth {
            ShareAuth::Open => auth_options.open = true,
            ShareAuth::Password { .. } => auth_options.password = true,
        }
    }

    if auth.as_ref().is_some_and(AuthUser::is_account) {
        for flow in share.account_flows.iter() {
            match flow.auth {
                ShareAuth::Open => auth_options.open = true,
                ShareAuth::Password { .. } => auth_options.password = true,
            }
        }
    }

    let root_name = StdPath::new(&share.root_entry)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| share.root_entry.clone());

    resp.success(GetShareResponse {
        id: share.id,
        space: share.space,
        root_name,
        auth: auth_options,
    })
    .into_response()
}
