use std::collections::BTreeSet;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    auth::AuthUser,
    config::Config,
    guest::{
        token::{self, GuestToken, DEFAULT_GUEST_TOKEN_LIFETIME_DAYS},
        Guest,
    },
    jsend::JSendBuilder,
    share::{Grant, Share, ShareAuth},
    space::Space,
    user::Permission,
    web::shares_api::shares_get::ShareIdParam,
};

#[derive(Deserialize)]
pub struct AuthShareRequest {
    password: Option<String>,
}

#[derive(Serialize)]
struct AuthShareResponse {
    token: Option<String>,
    permissions: BTreeSet<Permission>,
    expires_at: Option<DateTime<Utc>>,
}

pub async fn handle_authn_share(
    State(cfg): State<Config>,
    space: Space,
    auth: Option<AuthUser>,
    Path(params): Path<ShareIdParam>,
    Json(req): Json<AuthShareRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let mut share = match Share::load(&space, &params.id).await {
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

    // todo: create a filtered list of flows for the user

    let password = req.password.unwrap_or_default();
    let permissions = {
        let has_password_flow = share
            .guest_flows
            .iter()
            .any(|flow| matches!(flow.auth, ShareAuth::Password { .. }));
        if !has_password_flow {
            return resp
                .status_code(StatusCode::FORBIDDEN)
                .fail("share has no password-protected guest access")
                .into_response();
        }

        match share.guest_flows.iter().find(|flow| {
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

    if let Some(account) = auth.as_ref().and_then(|user| user.as_account()) {
        share.account_allowlist.insert(
            account.name.clone(),
            Grant {
                principal_id: account.name.clone(),
                permissions: permissions.clone(),
                created_at: Utc::now(),
            },
        );
        if let Err(err) = share.save(&space).await {
            warn!("could not record share member for {}: {:#}", share.id, err);
            return resp.internal_error().into_response();
        }

        return resp
            .success(AuthShareResponse {
                token: None,
                permissions,
                expires_at: None,
            })
            .into_response();
    }

    let guest = Guest::new();
    share.guests_allowlist.insert(
        guest.id.clone(),
        Grant {
            principal_id: guest.id.clone(),
            permissions: permissions.clone(),
            created_at: Utc::now(),
        },
    );
    if let Err(err) = share.save(&space).await {
        warn!("could not record share guest for {}: {:#}", share.id, err);
        return resp.internal_error().into_response();
    }

    let guest_token = GuestToken::new(&guest.id, Duration::days(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS));
    let expires_at = guest_token.expires_at();
    let encoded = match token::encode_token(&cfg.token_encryption_key, &guest_token) {
        Ok(encoded) => encoded,
        Err(err) => {
            warn!("could not encode guest token for {}: {:#}", share.id, err);
            return resp.internal_error().into_response();
        }
    };

    resp.success(AuthShareResponse {
        token: Some(encoded),
        permissions,
        expires_at: Some(expires_at),
    })
    .into_response()
}
