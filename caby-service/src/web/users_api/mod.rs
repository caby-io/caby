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
    user::user_init::{InitFileCode, InitMethod, UserInitFile},
    web::auth_api::auth_login::LoginRequest,
};

#[derive(Deserialize)]
pub struct UserInitParams {
    pub user: String,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserInitState {
    Ready,
    InProgress,
    Completed,
}

#[derive(Serialize)]
pub struct UserInitResponse {
    pub init_state: UserInitState,
}

// Returns the User's init state
pub async fn handle_get_user_init(
    State(cfg): State<Config>,
    path_params: Path<UserInitParams>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    if !cfg.users.contains_key(&path_params.user) {
        return resp.fail("bad request").into_response();
    }

    // check if user is already initialized
    let user_dir_exists = match try_exists(&cfg.users_path.join(&path_params.user)).await {
        Ok(e) => e,
        Err(err) => {
            error!("could not lookup user dir: {}", err);
            return resp.internal_error().into_response();
        }
    };

    if (!user_dir_exists) {
        return resp
            .success(UserInitResponse {
                init_state: UserInitState::Ready,
            })
            .into_response();
    }

    // For now the lack of the init file will indicate an initialized user, this may change in the future
    // todo: we should actually look for the password file or profile being complete
    let init_file_exists =
        match try_exists(&cfg.users_path.join(&path_params.user).join("init.yaml")).await {
            Ok(e) => e,
            Err(err) => {
                error!("could not lookup user init file: {}", err);
                return resp.internal_error().into_response();
            }
        };

    if (!init_file_exists) {
        return resp
            .success(UserInitResponse {
                init_state: UserInitState::Completed,
            })
            .into_response();
    }

    return resp
        .success(UserInitResponse {
            init_state: UserInitState::InProgress,
        })
        .into_response();
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
    path_params: Path<UserInitParams>,
    Json(req): Json<StartUserInitRequest>,
) -> Response {
    let resp = jsend::JSendBuilder::new();

    if !cfg.users.contains_key(&path_params.user) {
        return resp.fail("bad request").into_response();
    }

    let user_dir_exists = match try_exists(&cfg.users_path.join(&path_params.user)).await {
        Ok(e) => e,
        Err(err) => {
            error!("could not lookup user dir: {}", err);
            return resp.internal_error().into_response();
        }
    };

    if (user_dir_exists) {
        return resp.fail("bad request").into_response();
    }

    let init_file_exists =
        match try_exists(&cfg.users_path.join(&path_params.user).join("init.yaml")).await {
            Ok(e) => e,
            Err(err) => {
                error!("could not lookup user init file: {}", err);
                return resp.internal_error().into_response();
            }
        };

    if (init_file_exists) {
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
