use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use jiff::Timestamp;
use serde::Serialize;
use tracing::warn;

use crate::{config::Config, jsend::JSendBuilder, sse::sse_token, web::extractors::RequireAccount};

#[derive(Serialize)]
struct RegisterSseResponse {
    token: String,
    expires_at: Timestamp,
}

pub async fn handle_register_sse(
    State(cfg): State<Config>,
    RequireAccount(account): RequireAccount,
) -> Response {
    let resp = JSendBuilder::new();

    let (token, payload) = match sse_token::generate_token(&cfg, &account.name) {
        Ok(t) => t,
        Err(err) => {
            warn!("could not generate sse token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    let expires_at = match payload.expires_at() {
        Ok(t) => t,
        Err(err) => {
            warn!("could not compute sse token expiry: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    resp.success(RegisterSseResponse { token, expires_at })
        .into_response()
}
