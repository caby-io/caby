use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::{config::Config, ctx::Ctx, error::Result, jsend};

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
    ctx: Result<Ctx>,
    extract::Json(req): extract::Json<LoginRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    resp.success(LoginResponse {
        login_token: "token".to_string(),
    })
    .into_response()
}
