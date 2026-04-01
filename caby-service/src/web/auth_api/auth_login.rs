use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{config::Config, jsend};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    login_token: String,
}

pub async fn handle_login(
    State(cfg): State<Config>,
    extract::Json(req): extract::Json<LoginRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    let Some(user) = cfg.users.get(&req.login) else {
        return resp.fail("bad login").into_response();
    };

    resp.success(LoginResponse {
        login_token: "token".to_string(),
    })
    .into_response()
}
