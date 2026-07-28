use std::str::FromStr;

use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts, OptionalFromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::{AuthUser, Token, User},
    config::Config,
    guest::{token, Guest},
    jsend::JSendBuilder,
    user::Account,
    web::headers::HEADER_CABY_USER_NAME,
};

#[derive(Serialize)]
pub struct UnauthorizedResponse<'a> {
    pub reason: &'a str,
}

#[derive(Deserialize)]
struct GuestTokenQuery {
    token: Option<String>,
}

fn unauthorized() -> Response {
    JSendBuilder::new()
        .status_code(StatusCode::UNAUTHORIZED)
        .fail(UnauthorizedResponse {
            reason: "unauthorized",
        })
        .into_response()
}

async fn find_session(
    cfg: &Config,
    token: &str,
    user_name: Option<&str>,
) -> crate::Result<(Token, Account)> {
    let cfg_rtm = cfg.runtime.load();

    if let Some(name) = user_name {
        let user = cfg_rtm
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
    for (_, user) in cfg_rtm.users.iter() {
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
    Err(anyhow!("token not found"))
}

async fn resolve<S>(parts: &mut Parts, state: &S) -> Result<Option<AuthUser>, Response>
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    let cfg = Config::from_ref(state);

    let bearer = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    if let Some(token_str) = bearer {
        let user_name = parts
            .headers
            .get(&HEADER_CABY_USER_NAME)
            .and_then(|h| h.to_str().ok());

        let (session, account) = find_session(&cfg, token_str, user_name)
            .await
            .map_err(|err| {
                warn!("could not authorize user token: {:#}", err);
                unauthorized()
            })?;

        if session.is_expired() {
            warn!("user authenticated with an expired token: {}", account.name);
            return Err(unauthorized());
        }

        return Ok(Some(AuthUser {
            token: token_str.to_owned(),
            user: User::Account(account),
        }));
    }

    let guest_token = Query::<GuestTokenQuery>::from_request_parts(parts, state)
        .await
        .ok()
        .and_then(|Query(query)| query.token);

    if let Some(guest_token) = guest_token {
        let decoded =
            token::decode_token(&cfg.token_encryption_key, &guest_token).map_err(|err| {
                warn!("could not decode guest token: {:#}", err);
                unauthorized()
            })?;

        if decoded.is_expired() {
            return Err(unauthorized());
        }

        return Ok(Some(AuthUser {
            token: guest_token,
            user: User::Guest(Guest::from(&decoded)),
        }));
    }

    Ok(None)
}

impl<S> FromRequestParts<S> for AuthUser
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match resolve(parts, state).await? {
            Some(auth) => Ok(auth),
            None => Err(unauthorized()),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for AuthUser
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        resolve(parts, state).await
    }
}

pub struct RequireAccount;

impl<S> FromRequestParts<S> for RequireAccount
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match resolve(parts, state).await? {
            Some(auth) if auth.as_account().is_some() => Ok(RequireAccount),
            Some(_) => Err(JSendBuilder::new()
                .status_code(StatusCode::FORBIDDEN)
                .fail(UnauthorizedResponse {
                    reason: "account required",
                })
                .into_response()),
            None => Err(unauthorized()),
        }
    }
}
