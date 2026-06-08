use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{auth::oidc::OidcClient, config::Config, jsend};

#[derive(Serialize)]
pub struct AuthInfoResponse {
    passwords_enabled: bool,
    oidc_enabled: bool,
}

pub async fn handle_auth_info(
    State(cfg): State<Config>,
    State(oidc_client): State<Option<Arc<OidcClient>>>,
) -> Response {
    jsend::JSendBuilder::new()
        .success(AuthInfoResponse {
            passwords_enabled: cfg.auth.passwords.enabled,
            oidc_enabled: oidc_client.is_some(),
        })
        .into_response()
}
