use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokio::fs;
use tracing::error;

use crate::{auth::AuthUser, config::Config, jsend};

#[derive(Serialize)]
pub struct LogoutResponse<'a> {
    user: &'a str,
}

pub async fn handle_logout(State(_cfg): State<Config>, auth: AuthUser) -> Response {
    let Some(account) = auth.as_account() else {
        return jsend::JSendBuilder::new()
            .status_code(StatusCode::FORBIDDEN)
            .fail("only account users can log out")
            .into_response();
    };

    let session_path = account.path.join(format!("session_{}", auth.token));

    if let Err(err) = fs::remove_file(&session_path).await {
        error!("could not delete session file: {:#}", err);
        return jsend::JSendBuilder::new().internal_error().into_response();
    }

    jsend::JSendBuilder::new()
        .success(LogoutResponse {
            user: &account.name,
        })
        .into_response()
}
