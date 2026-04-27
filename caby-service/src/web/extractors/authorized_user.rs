use std::str::FromStr;

use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokio::fs;
use tracing::warn;

use crate::{
    auth::{AuthorizedUser, Token},
    config::Config,
    jsend::JSendBuilder,
    user::User,
};

const HEADER_CABY_USER: &str = "Caby-User-Name";

#[derive(Serialize)]
pub struct UnauthorizedResponse<'a> {
    pub reason: &'a str,
}

async fn find_session(
    cfg: &Config,
    token: &str,
    user_name: Option<&str>,
) -> crate::Result<(Token, User)> {
    if let Some(name) = user_name {
        let user = cfg
            .users
            .get(&name.to_lowercase())
            .ok_or_else(|| anyhow!("user does not exist: {}", name))?;

        let session_content = fs::read_to_string(user.path.join(format!("session_{}", token)))
            .await
            .map_err(|err| {
                anyhow!(err).context(format!("could not read session file for user: {}", name))
            })?;

        return Ok((Token::from_str(&session_content)?, user.into()));
    }

    // This is intentionally unoptimized and slow. We should encode the user name into the token so that we don't need to do this at all
    for (_, user) in cfg.users.iter() {
        let session_file = user.path.join(format!("session_{}", token));

        if !fs::try_exists(session_file).await.map_err(|err| {
            anyhow!(err).context(format!(
                "could not lookup session file for user: {}",
                user.name
            ))
        })? {
            continue;
        };

        let session_content = fs::read_to_string(user.path.join(format!("session_{}", token)))
            .await
            .map_err(|err| {
                anyhow!(err).context(format!(
                    "could not read session file for user {}",
                    user.name
                ))
            })?;

        return Ok((Token::from_str(&session_content)?, user.into()));
    }

    // todo: return a specific error so we can match on it
    return Err(anyhow!("token not found"));
}

impl<S> FromRequestParts<S> for AuthorizedUser
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cfg = Config::from_ref(state);

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .ok_or_else(|| {
                JSendBuilder::new()
                    .status_code(StatusCode::UNAUTHORIZED)
                    .fail(UnauthorizedResponse {
                        reason: "unauthorized",
                    })
                    .into_response()
            })?;

        let token_str = auth_header.strip_prefix("Bearer ").ok_or(
            JSendBuilder::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .fail(UnauthorizedResponse {
                    reason: "unauthorized",
                })
                .into_response(),
        )?;

        let user = parts
            .headers
            .get(HEADER_CABY_USER)
            .and_then(|h| h.to_str().ok());

        let (token, user) = match find_session(&cfg, token_str, user).await {
            Ok(t) => t,
            Err(err) => {
                warn!("could not authorize user token: {:#}", err);
                return Err(JSendBuilder::new()
                    .status_code(StatusCode::UNAUTHORIZED)
                    .fail(UnauthorizedResponse {
                        reason: "unauthorized",
                    })
                    .into_response());
            }
        };

        if token.is_expired() {
            warn!("user authenticated with an expired token: {}", user.name);
            return Err(JSendBuilder::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .fail(UnauthorizedResponse {
                    reason: "unauthorized",
                })
                .into_response());
        }

        Ok(AuthorizedUser { token, user })
    }
}
