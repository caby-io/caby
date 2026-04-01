use axum::{
    extract::{FromRef, FromRequestParts, Path, RawPathParams},
    http::request::Parts,
    response::{IntoResponse, Response},
    Extension, RequestPartsExt,
};
use serde::Serialize;
use tokio::fs::try_exists;
use tracing::error;

use crate::{
    config::Config, jsend::JSendBuilder, user::user_init::UserInitState,
    web::users_api::UserInitParams,
};

impl<S> FromRequestParts<S> for UserInitState
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let resp = JSendBuilder::new();
        let cfg = Config::from_ref(state);

        let Ok(Path(path_params)) = parts.extract::<Path<UserInitParams>>().await else {
            // todo: log specific err
            return Err(resp.fail("bad request").into_response());
        };

        if !cfg.users.contains_key(&path_params.user) {
            return Err(resp.fail("bad request").into_response());
        }

        // check if user is already initialized
        let user_dir_exists = match try_exists(&cfg.users_path.join(&path_params.user)).await {
            Ok(e) => e,
            Err(err) => {
                error!("could not lookup user dir: {}", err);
                return Err(resp.internal_error().into_response());
            }
        };

        if (!user_dir_exists) {
            return Ok(UserInitState::Ready);
        }

        // For now the lack of the init file will indicate an initialized user, this may change in the future
        // todo: we should actually look for the password file or profile being complete
        let init_file_exists =
            match try_exists(&cfg.users_path.join(&path_params.user).join("init.yaml")).await {
                Ok(e) => e,
                Err(err) => {
                    error!("could not lookup user init file: {}", err);
                    return Err(resp.internal_error().into_response());
                }
            };

        if (!init_file_exists) {
            return Ok(UserInitState::Completed);
        }

        return Ok(UserInitState::InProgress);
    }
}
