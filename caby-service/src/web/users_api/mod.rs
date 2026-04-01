use std::str::FromStr;

use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tokio::fs::try_exists;
use tracing::{error, info};

use crate::{
    config::Config,
    jsend,
    user::user_init::{InitMethod, UserInitFile, UserInitState},
};

#[derive(Deserialize)]
pub struct UserInitParams {
    pub user: String,
}

#[derive(Serialize)]
pub struct UserInitResponse {
    pub init_state: UserInitState,
}

// Returns the User's init state
pub async fn handle_get_user_init(
    State(cfg): State<Config>,
    init_state: UserInitState,
    path_params: Path<UserInitParams>,
) -> Response {
    jsend::JSendBuilder::new()
        .success(UserInitResponse { init_state })
        .into_response()
}

#[derive(Deserialize)]
pub struct StartUserInitRequest {
    pub method: String,
}

// #[derive(Serialize)]
// struct StartUserInitResponse {

// }

pub async fn handle_start_user_init(
    State(cfg): State<Config>,
    init_state: UserInitState,
    path_params: Path<UserInitParams>,
    Json(req): Json<StartUserInitRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    if !matches!(init_state, UserInitState::Ready) {
        return resp.fail("bad request").into_response();
    }

    let Ok(method) = InitMethod::from_str(&req.method) else {
        return resp.fail("bad request").into_response();
    };

    let mut init_file = UserInitFile {
        method: method.clone(),
        locked_until: None,
        code: None,
        email: None,
    };

    let init_file = match method {
        InitMethod::Code => {
            let init_file = UserInitFile::new_code();
            let code = init_file.code.as_ref().unwrap().value.clone();
            info!("activation code for {}: {}", path_params.user, code);
            init_file
        }
        InitMethod::Email => UserInitFile::new_email(),
    };

    let user_dir = cfg.users_path.join(&path_params.user);
    if let Err(err) = tokio::fs::create_dir_all(&user_dir).await {
        error!("could not create user dir: {}", err);
        return resp.internal_error().into_response();
    }

    let init_path = user_dir.join("init.yaml");
    if let Err(err) = init_file.write(&init_path).await {
        error!("could not write init file: {}", err);
        return resp.internal_error().into_response();
    }

    return resp.success("success").into_response();
}
