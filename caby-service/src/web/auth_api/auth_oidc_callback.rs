use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use openidconnect::{core::CoreClient, AuthorizationCode, Nonce, PkceCodeVerifier, TokenResponse};
use serde::Deserialize;
use tracing::error;
use url::form_urlencoded;

use crate::{
    auth::oidc::{oidc_auth_code_flow::AuthCodeFlow, oidc_user::provision_user, OidcClient},
    config::Config,
    jsend::JSendBuilder,
    user::User,
};

#[derive(Deserialize)]
pub struct OidcCallbackQuery {
    pub code: String,
    pub state: String,
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

    let Some(oidc_cfg) = cfg.auth.oidc.as_ref() else {
        error!("oidc: callback hit but config is missing");
        return JSendBuilder::new().internal_error().into_response();
    };
    let post_login_redirect = oidc_cfg.post_login_redirect.clone();

    let flow = match AuthCodeFlow::take(&cfg.home_path, &query.state).await {
        Ok(f) => f,
        Err(err) => {
            error!("oidc: invalid flow state: {:#}", err);
            return redirect_with_error(&post_login_redirect, "invalid or expired OIDC state");
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
            return redirect_with_error(&post_login_redirect, "OIDC token exchange failed");
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
            return redirect_with_error(&post_login_redirect, "OIDC token exchange failed");
        }
    };

    let Some(id_token) = token_response.id_token() else {
        error!("oidc: token response missing id_token");
        return redirect_with_error(&post_login_redirect, "OIDC id_token missing");
    };

    let verifier = core.id_token_verifier();
    let nonce = Nonce::new(flow.nonce);
    let claims = match id_token.claims(&verifier, &nonce) {
        Ok(c) => c,
        Err(err) => {
            error!("oidc: id_token verification failed: {:#}", err);
            return redirect_with_error(&post_login_redirect, "OIDC id_token verification failed");
        }
    };

    let user: User = match cfg.find_user(claims.subject().as_str()) {
        Some(uc) => (&uc).into(),
        None => match provision_user(&cfg, claims).await {
            Ok(u) => u,
            Err(err) => {
                error!("oidc: provision failed: {:#}", err);
                return redirect_with_error(&post_login_redirect, "could not provision user");
            }
        },
    };

    let token = match user.create_session().await {
        Ok(t) => t,
        Err(err) => {
            error!(
                "oidc: could not create session for {}: {:#}",
                user.name, err
            );
            return redirect_with_error(&post_login_redirect, "could not create session");
        }
    };

    let fragment = form_urlencoded::Serializer::new(String::new())
        .append_pair("login_token", &token.value)
        .append_pair("user", &user.name)
        .append_pair("expires_at", &token.expires_at.to_rfc3339())
        .finish();

    Redirect::temporary(&format!("{}#{}", post_login_redirect, fragment)).into_response()
}

fn redirect_with_error(post_login_redirect: &str, message: &str) -> Response {
    let fragment = form_urlencoded::Serializer::new(String::new())
        .append_pair("error", message)
        .finish();
    Redirect::temporary(&format!("{}#{}", post_login_redirect, fragment)).into_response()
}
