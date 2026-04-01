use axum::{
    extract::{FromRef, FromRequestParts, Path},
    http::request::Parts,
    response::{IntoResponse, Response}, RequestPartsExt,
};

use crate::{config::Config, jsend::JSendBuilder, user::User, web::users_api::UserPathParams};

impl<S> FromRequestParts<S> for User
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cfg = Config::from_ref(state);

        let Ok(Path(path_params)) = parts.extract::<Path<UserPathParams>>().await else {
            // todo: log specific err
            return Err(JSendBuilder::new().fail("bad request").into_response());
        };

        let Some(user_config) = cfg.users.get(&path_params.user) else {
            // todo: log specific err
            return Err(JSendBuilder::new().fail("bad request").into_response());
        };

        Ok(user_config.into())
    }
}
