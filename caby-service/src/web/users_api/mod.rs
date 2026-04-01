
use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{config::Config, jsend, user::User};

#[derive(Deserialize)]
pub struct UserPathParams {
    pub user: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivateUserAction {
    ValidateToken,
    Activate,
}

#[derive(Deserialize)]
pub struct ActivateUserRequest {
    pub action: ActivateUserAction,
    pub token: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
struct ValidateTokenResponse {
    pub is_token_valid: bool,
}

#[derive(Serialize)]
struct ActivateUserResponse {
    pub is_user_activated: bool,
}

pub async fn handle_activate_user(
    State(cfg): State<Config>,
    user: User,
    path_params: Path<UserPathParams>,
    Json(req): Json<ActivateUserRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    let is_activated = match user.is_activated().await {
        Ok(a) => a,
        Err(err) => {
            error!(
                "could not check if user {} is activated: {}",
                &user.name, err
            );
            return resp.internal_error().into_response();
        }
    };

    if is_activated {
        return resp.fail("bad request").into_response();
    };

    if matches!(req.action, ActivateUserAction::ValidateToken) {
        // todo: validate token
    }

    let user_dir = cfg.users_path.join(&path_params.user);
    if let Err(err) = tokio::fs::create_dir_all(&user_dir).await {
        error!("could not create user dir: {}", err);
        return resp.internal_error().into_response();
    }

    let init_path = user_dir.join("init.yaml");

    return resp.success("success").into_response();
}
