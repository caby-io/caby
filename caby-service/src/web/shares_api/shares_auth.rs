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
        token::{self, GuestGrant, GuestToken, DEFAULT_GUEST_TOKEN_LIFETIME_DAYS},
        Guest,
    },
    jsend::JSendBuilder,
    share::{fingerprint, Share, ShareAuth},
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

pub async fn handle_auth_share(
    State(cfg): State<Config>,
    space: Space,
    auth: Option<AuthUser>,
    Path(params): Path<ShareIdParam>,
    Json(req): Json<AuthShareRequest>,
) -> Response {
    let mut share = match Share::load(&space, &params.id).await {
        Ok(Some(share)) => share,
        Ok(None) => {
            return JSendBuilder::new()
                .status_code(StatusCode::NOT_FOUND)
                .fail("share not found")
                .into_response()
        }
        Err(err) => {
            warn!("could not load share {}: {:#}", params.id, err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    if share.is_expired() {
        return JSendBuilder::new()
            .status_code(StatusCode::GONE)
            .fail("share has expired")
            .into_response();
    }

    let Some(flow) = share.guest_flow.clone() else {
        return JSendBuilder::new()
            .status_code(StatusCode::FORBIDDEN)
            .fail("share has no guest access")
            .into_response();
    };

    let hash = match &flow.auth {
        ShareAuth::Open => {
            return JSendBuilder::new()
                .success(AuthShareResponse {
                    token: None,
                    permissions: flow.permissions,
                    expires_at: None,
                })
                .into_response();
        }
        ShareAuth::Password { hash } => hash,
    };

    let password = req.password.unwrap_or_default();
    let verified = match flow.auth.try_verify(&password) {
        Ok(verified) => verified,
        Err(err) => {
            warn!(
                "could not verify share password for {}: {:#}",
                share.id, err
            );
            return JSendBuilder::new().internal_error().into_response();
        }
    };
    if !verified {
        return JSendBuilder::new()
            .status_code(StatusCode::UNAUTHORIZED)
            .fail("incorrect password")
            .into_response();
    }

    if let Some(account) = auth.as_ref().and_then(|user| user.as_account()) {
        if !share.is_member(&account.name) {
            share.members.push(account.name.clone());
            if let Err(err) = share.save(&space).await {
                warn!("could not record share member for {}: {:#}", share.id, err);
                return JSendBuilder::new().internal_error().into_response();
            }
        }

        return JSendBuilder::new()
            .success(AuthShareResponse {
                token: None,
                permissions: flow.permissions,
                expires_at: None,
            })
            .into_response();
    }

    let guest = Guest::new();
    let guest_token = GuestToken::new(
        &guest.id,
        vec![GuestGrant {
            space: space.name.clone(),
            share_id: share.id.clone(),
            pw_fingerprint: fingerprint(hash),
        }],
        Duration::days(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS),
    );
    let expires_at = guest_token.expires_at();
    let encoded = match token::encode_token(&cfg.token_encryption_key, &guest_token) {
        Ok(encoded) => encoded,
        Err(err) => {
            warn!("could not encode guest token for {}: {:#}", share.id, err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    JSendBuilder::new()
        .success(AuthShareResponse {
            token: Some(encoded),
            permissions: flow.permissions,
            expires_at: Some(expires_at),
        })
        .into_response()
}
