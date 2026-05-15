use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use openidconnect::{core::CoreClient, AuthorizationCode, Nonce, PkceCodeVerifier, TokenResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    auth::oidc::{flow_state::FlowState, OidcClient},
    config::Config,
    jsend::JSendBuilder,
};

#[derive(Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct OidcCallbackResponse {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub name: Option<String>,
}

pub async fn handle_oidc_callback(
    State(cfg): State<Config>,
    State(oidc_client): State<Option<Arc<OidcClient>>>,
    Query(query): Query<OidcCallbackQuery>,
) -> Response {
    let Some(client) = oidc_client else {
        return JSendBuilder::new()
            .fail("OIDC is not enabled")
            .into_response();
    };

    let flow = match FlowState::take(&cfg.home_path, &query.state).await {
        Ok(f) => f,
        Err(err) => {
            error!("oidc: invalid flow state: {:#}", err);
            return JSendBuilder::new()
                .fail("invalid or expired OIDC state")
                .into_response();
        }
    };

    let core = CoreClient::from_provider_metadata(
        client.metadata.clone(),
        client.client_id.clone(),
        Some(client.client_secret.clone()),
    )
    .set_redirect_uri(client.redirect_uri.clone());

    let token_request = match core.exchange_code(AuthorizationCode::new(query.code)) {
        Ok(r) => r,
        Err(err) => {
            error!("oidc: could not build token exchange request: {:#}", err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let token_response = match token_request
        .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
        .request_async(&client.http)
        .await
    {
        Ok(r) => r,
        Err(err) => {
            error!("oidc: token exchange failed: {:#}", err);
            return JSendBuilder::new()
                .fail("OIDC token exchange failed")
                .into_response();
        }
    };

    let Some(id_token) = token_response.id_token() else {
        error!("oidc: token response missing id_token");
        return JSendBuilder::new().internal_error().into_response();
    };

    let verifier = core.id_token_verifier();
    let nonce = Nonce::new(flow.nonce);
    let claims = match id_token.claims(&verifier, &nonce) {
        Ok(c) => c,
        Err(err) => {
            error!("oidc: id_token verification failed: {:#}", err);
            return JSendBuilder::new()
                .fail("OIDC id_token verification failed")
                .into_response();
        }
    };

    JSendBuilder::new()
        .success(OidcCallbackResponse {
            issuer: claims.issuer().to_string(),
            subject: claims.subject().to_string(),
            email: claims.email().map(|e| e.to_string()),
            preferred_username: claims.preferred_username().map(|u| u.to_string()),
            name: claims
                .name()
                .and_then(|n| n.get(None))
                .map(|n| n.to_string()),
        })
        .into_response()
}
