use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokio::fs;
use tracing::error;

use crate::{auth::AuthorizedUser, config::Config, jsend};

#[derive(Serialize)]
pub struct LogoutResponse<'a> {
    user: &'a str,
}

pub async fn handle_logout(State(cfg): State<Config>, authorized_user: AuthorizedUser) -> Response {
    let resp = jsend::JSendBuilder::new();

    let session_path = authorized_user
        .user
        .path
        .join(format!("session_{}", authorized_user.token.value));

    if let Err(err) = fs::remove_file(&session_path).await {
        error!("could not delete session file: {:#}", err);
        return resp.internal_error().into_response();
    }

    resp.success(LogoutResponse {
        user: &authorized_user.user.name,
    })
    .into_response()
}
