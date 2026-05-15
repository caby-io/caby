use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient},
    CsrfToken, Nonce, PkceCodeChallenge, Scope,
};
use tracing::error;

use crate::{
    auth::oidc::{flow_state::FlowState, OidcClient},
    config::Config,
    jsend::JSendBuilder,
};

pub async fn handle_oidc_login(
    State(cfg): State<Config>,
    State(oidc_client): State<Option<Arc<OidcClient>>>,
) -> Response {
    let Some(client) = oidc_client else {
        return JSendBuilder::new()
            .fail("OIDC is not enabled")
            .into_response();
    };

    let Some(oidc_cfg) = cfg.auth.oidc.as_ref() else {
        error!("oidc: missing config");
        return JSendBuilder::new().internal_error().into_response();
    };

    let core = CoreClient::from_provider_metadata(
        client.metadata.clone(),
        client.client_id.clone(),
        Some(client.client_secret.clone()),
    )
    .set_redirect_uri(client.redirect_uri.clone());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token, nonce) = core
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scopes(oidc_cfg.extra_scopes.iter().map(|s| Scope::new(s.clone())))
        .set_pkce_challenge(pkce_challenge)
        .url();

    if let Err(err) = FlowState::write(
        &cfg.home_path,
        csrf_token.secret(),
        pkce_verifier.secret(),
        nonce.secret(),
    )
    .await
    {
        error!("oidc: could not write flow state: {:#}", err);
        return JSendBuilder::new().internal_error().into_response();
    }

    Redirect::temporary(auth_url.as_str()).into_response()
}
