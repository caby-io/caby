use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts, Query},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio::fs;
use tracing::warn;

use crate::{
    config::Config, jsend::JSendBuilder, user::User,
    web::extractors::authorized_user::UnauthorizedResponse,
};

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

// todo: move to another module
pub struct DownloadUser {
    pub token: String,
    pub user: User,
}

// todo: we need to encode data into the token to speed up this lookup
async fn find_download_user(cfg: &Config, token: &str) -> crate::Result<DownloadUser> {
    // This is intentionally unoptimized and slow. We should encode the user name into the token so that we don't need to do this at all
    for (_, user) in cfg.users.iter() {
        let download_file = user.path.join(format!("download_{}", token));

        if !fs::try_exists(download_file).await.map_err(|err| {
            anyhow!(err).context(format!(
                "could not lookup download file for user: {}",
                user.name
            ))
        })? {
            continue;
        };

        // todo: open file and build download token

        return Ok(DownloadUser {
            token: token.to_string(),
            user: user.into(),
        });
    }

    // todo: return a specific error so we can match on it
    return Err(anyhow!("token not found"));
}

impl<S> FromRequestParts<S> for DownloadUser
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cfg = Config::from_ref(state);

        let unauthorized_resp = || {
            JSendBuilder::new()
                .status_code(StatusCode::UNAUTHORIZED)
                .fail(UnauthorizedResponse {
                    reason: "unauthorized",
                })
                .into_response()
        };

        let Query(TokenQuery { token }) = Query::<TokenQuery>::from_request_parts(parts, state)
            .await
            .map_err(|err| {
                // todo: warn message
                unauthorized_resp()
            })?;
        let token = token.ok_or_else(unauthorized_resp)?;

        let download_user = match find_download_user(&cfg, &token).await {
            Ok(u) => u,
            Err(err) => {
                warn!("could not authorize download token: {:#}", err);
                return Err(unauthorized_resp());
            }
        };

        Ok(download_user)
    }
}
