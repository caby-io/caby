use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use tracing::warn;

use crate::{
    auth::{AuthUser, User},
    config::Config,
    guest::{
        token::{self, GuestToken, DEFAULT_GUEST_TOKEN_LIFETIME_DAYS},
        Guest,
    },
    jsend,
};

#[derive(Serialize)]
pub struct GuestTokenResponse {
    guest_token: String,
    expires_at: DateTime<Utc>,
}

pub async fn handle_create_guest(State(cfg): State<Config>) -> Response {
    let resp = jsend::JSendBuilder::new();

    // todo: gate creation of guests based on some rate-limit

    let guest = Guest::new();
    let guest_token = GuestToken::new(&guest.id, Duration::days(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS));
    let expires_at = guest_token.expires_at();

    let encoded = match token::encode_token(&cfg.token_encryption_key, &guest_token) {
        Ok(encoded) => encoded,
        Err(err) => {
            warn!("could not encode guest token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    resp.success(GuestTokenResponse {
        guest_token: encoded,
        expires_at,
    })
    .into_response()
}

pub async fn handle_refresh_guest(State(cfg): State<Config>, auth: AuthUser) -> Response {
    let resp = jsend::JSendBuilder::new();

    let User::Guest(guest) = &auth.user else {
        return resp
            .status_code(StatusCode::BAD_REQUEST)
            .fail("only guest sessions can be refreshed")
            .into_response();
    };

    let guest_token = GuestToken::new(&guest.id, Duration::days(DEFAULT_GUEST_TOKEN_LIFETIME_DAYS));
    let expires_at = guest_token.expires_at();

    let encoded = match token::encode_token(&cfg.token_encryption_key, &guest_token) {
        Ok(encoded) => encoded,
        Err(err) => {
            warn!("could not encode guest token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    resp.success(GuestTokenResponse {
        guest_token: encoded,
        expires_at,
    })
    .into_response()
}
