use std::{collections::BTreeSet, sync::Arc};

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    auth::{AuthUser, User},
    config::Config,
    controller::PathLocks,
    jsend::JSendBuilder,
    share::{load_state, Share, ShareAccessFlow, ShareAuth},
    user::Permission,
    web::shares_api::ShareIdParam,
};

#[derive(Deserialize)]
pub struct PasswordAuthShareRequest {
    password: Option<String>,
}

#[derive(Serialize)]
struct AuthShareResponse {
    permissions: BTreeSet<Permission>,
}

pub async fn handle_password_auth_share(
    State(cfg): State<Config>,
    auth: AuthUser,
    Path(params): Path<ShareIdParam>,
    State(locks): State<Arc<PathLocks>>,
    Json(req): Json<PasswordAuthShareRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let (space, spec_path) = match Share::resolve(&cfg, &params.id).await {
        Ok(Some((space, share))) => (space, share.spec_path),
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

    let guard = locks
        .acquire(&space.name, std::path::Path::new(&spec_path))
        .await;

    let mut share = match load_state(&space, std::path::Path::new(&spec_path)).await {
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

    let password = req.password.unwrap_or_default();
    let permissions = {
        let applicable: Vec<&ShareAccessFlow> = match &auth.user {
            User::Account(_) => share
                .account_flows
                .iter()
                .chain(share.guest_flows.iter())
                .collect(),
            User::Guest(_) => share.guest_flows.iter().collect(),
        };

        let has_password_flow = applicable
            .iter()
            .any(|flow| matches!(flow.auth, ShareAuth::Password { .. }));
        if !has_password_flow {
            return resp
                .status_code(StatusCode::FORBIDDEN)
                .fail("share has no password-protected access")
                .into_response();
        }

        match applicable.iter().find(|flow| {
            matches!(flow.auth, ShareAuth::Password { .. })
                && flow.auth.try_verify(&password).unwrap_or(false)
        }) {
            Some(flow) => flow.permissions.clone(),
            None => {
                return resp
                    .status_code(StatusCode::UNAUTHORIZED)
                    .fail("incorrect password")
                    .into_response();
            }
        }
    };

    share.grant(&auth.user, permissions.clone());

    if let Err(err) = share.save(&cfg.shares_path, &space, &guard).await {
        warn!("could not record share grant for {}: {:#}", share.id, err);
        return resp.internal_error().into_response();
    }

    resp.success(AuthShareResponse { permissions })
        .into_response()
}
