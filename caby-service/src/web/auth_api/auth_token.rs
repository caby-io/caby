use std::io::ErrorKind;

use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, try_exists};
use tracing::{error, warn};

use crate::{
    config::Config,
    jsend::{self, JSendBuilder},
    user::{try_hash_password, User},
    Result,
};

const MAX_ACTIVATION_ATTEMPTS: i64 = 5;

#[derive(Deserialize)]
pub struct LookupTokenRequest {
    activation_token: String,
}

#[derive(Serialize)]
pub struct LookupTokenResponse {
    username: String,
}

pub async fn handle_token_lookup(
    State(cfg): State<Config>,
    Json(req): Json<LookupTokenRequest>,
) -> Response {
    let resp = JSendBuilder::new();
    // todo: validate token before performing lookup

    // lookup user by the provided activation token
    let Some(user_config) = cfg
        .users
        .values()
        .find(|u| u.activation_token.as_deref() == Some(&req.activation_token))
    else {
        return resp.fail("bad request").into_response();
    };

    // check if the user is already activated
    let user: User = user_config.into();
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
    }

    // respond with the username
    resp
        .success(LookupTokenResponse {
            username: user.name,
        })
        .into_response()
}

// todo: move to user package
pub async fn try_check_activation_attempts(user: &User) -> Result<i64> {
    let activation_attempts_path = &user.path.join("activation_attempts");
    let activation_attempts_exists = try_exists(activation_attempts_path)
        .await
        .map_err(|err| anyhow!(err).context("could not lookup activation_attempts file"))?;

    if (!activation_attempts_exists) {
        return Ok(0);
    }

    let content = fs::read_to_string(activation_attempts_path)
        .await
        .map_err(|err| anyhow!(err).context("could not read from activation_attempts file"))?;

    content
        .parse()
        .map_err(|err| anyhow!("could not parse activation_attempts as i64: {}", err))
}

pub async fn try_set_activation_attempts(user: &User, attempts: i64) -> Result<()> {
    if let Err(err) = fs::write(user.path.join("activation_attempts"), attempts.to_string()).await {
        return Err(anyhow!(err).context("could not write to activation_attempts file"));
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct ActivateUserRequest {
    pub activation_token: String,
    pub password: Option<String>,
}

#[derive(Serialize)]
struct ActivateUserResponse {
    pub is_user_activated: bool,
}

pub async fn handle_user_token_activation(
    State(cfg): State<Config>,
    Json(req): Json<ActivateUserRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    // lookup user by the provided activation token
    let Some(user_config) = cfg
        .users
        .values()
        .find(|u| u.activation_token.as_deref() == Some(&req.activation_token))
    else {
        return resp.fail("bad request").into_response();
    };

    // check if the user is already activated
    let user: User = user_config.into();
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
    }

    // Check activation attempts and return failure if we've exceeded the max
    let activation_attempts = match try_check_activation_attempts(&user).await {
        Ok(a) => a,
        Err(err) => {
            error!("could not check activation attempts: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    if activation_attempts > MAX_ACTIVATION_ATTEMPTS {
        warn!("activation attempts exceeded for user: {}", user.name);
        return resp.fail("bad request").into_response();
    }

    let Some(ref user_activation_token) = user.activation_token else {
        warn!("bad token for user: {}", user.name);
        if let Err(err) = try_set_activation_attempts(&user, activation_attempts + 1).await {
            error!("could not update activation_attempts file: {:#}", err);
            return resp.internal_error().into_response();
        }
        return resp.fail("bad request").into_response();
    };

    let Some(password) = req.password else {
        return resp.fail("missing password").into_response();
    };

    if &req.activation_token != user_activation_token {
        if let Err(err) = try_set_activation_attempts(&user, activation_attempts + 1).await {
            error!("could not update activation_attempts file: {:#}", err);
            return resp.internal_error().into_response();
        }
        return resp.fail("bad request").into_response();
    }

    if let Err(err) = fs::create_dir_all(&user.path).await {
        error!("could not create user dir: {:#}", err);
        return resp.internal_error().into_response();
    }

    let hashed_password = match try_hash_password(&password) {
        Ok(p) => p,
        Err(err) => {
            error!("{:#}", err);
            return resp.internal_error().into_response();
        }
    };

    if let Err(err) = fs::remove_file(&user.path.join("activation_attempts")).await {
        if err.kind() != ErrorKind::NotFound {
            error!("could not remove activation_attempts file: {:#}", err);
            return resp.internal_error().into_response();
        }
    };

    if let Err(err) = fs::write(&user.path.join("password"), &hashed_password).await {
        error!("could not write password file: {:#}", err);
        return resp.internal_error().into_response();
    }

    resp.success("account activated").into_response()
}
