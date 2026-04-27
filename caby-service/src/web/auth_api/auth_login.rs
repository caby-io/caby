use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{auth::Token, config::Config, jsend, user::User};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse<'a> {
    user: &'a str,
    login_token: Token,
}

pub async fn handle_login(
    State(cfg): State<Config>,
    extract::Json(req): extract::Json<LoginRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    let mut user_config = match cfg.users.get(&req.login.to_lowercase()) {
        Some(u) => u,
        None => {
            // todo: regex the login to see if it looks like an email before doing this expensive lookup
            let Some(user_config) = cfg.users.iter().map(|(_, u)| u).find(|u| {
                if let Some(email) = &u.email {
                    return email == &req.login;
                }
                return false;
            }) else {
                return resp.fail("invalid login or password").into_response();
            };
            user_config
        }
    };

    let user: User = user_config.into();

    let is_password = match user.is_password(&req.password).await {
        Ok(p) => p,
        Err(err) => {
            error!("could not check user password: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    if !is_password {
        warn!("wrong password for user: {}", user.name);
        return resp.fail("invalid login or password").into_response();
    }

    // temp
    let token = match user.create_session().await {
        Ok(t) => t,
        Err(err) => {
            error!("could not create user login session/token: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    resp.success(LoginResponse {
        user: &user.name,
        login_token: token,
    })
    .into_response()
}
