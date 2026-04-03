use std::str::FromStr;

use anyhow::anyhow;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tokio::fs;
use tracing::warn;

use crate::{auth::Token, config::Config, user::User};

const HEADER_CABY_USER: &str = "Caby-User-Name";
// const VALID_API_KEY: &str = "Bearer my_secret_api_key";

async fn find_session(
    cfg: &Config,
    token: &str,
    user_name: Option<&str>,
) -> crate::Result<(Token, User)> {
    if let Some(name) = user_name {
        let user = cfg
            .users
            .get(name)
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

pub async fn auth(
    State(cfg): State<Config>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token_str = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = req
        .headers()
        .get(HEADER_CABY_USER)
        .and_then(|h| h.to_str().ok());

    let (token, user) = match find_session(&cfg, token_str, user).await {
        Ok(t) => t,
        Err(err) => {
            warn!("could not authorize user token: {:#}", err);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // todo: check token expiry

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
