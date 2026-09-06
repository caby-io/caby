use axum::extract::Path;

use crate::{
    config::Config,
    jsend::{Fail, JSendBuilder},
    space::Space,
    web::files_api::files_list::{FilesPathParams, FILE_NOT_FOUND},
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};

impl<S> FromRequestParts<S> for Space
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = JSendBuilder<Fail<&'static str>>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cfg = Config::from_ref(state);

        let Ok(Path(path_params)) = parts.extract::<Path<FilesPathParams>>().await else {
            // todo: log specific err
            return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        };

        let cfg_rtm = cfg.runtime.load();
        let Some(space_config) = cfg_rtm.spaces.get(&path_params.space) else {
            // todo: log specific err
            return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        };

        Ok(space_config.into())
    }
}

pub struct WritableSpace(pub Space);

impl<S> FromRequestParts<S> for WritableSpace
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = JSendBuilder<Fail<&'static str>>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let space = parts.extract_with_state::<Space, S>(state).await?;
        if space.readonly {
            return Err(JSendBuilder::new()
                .status_code(StatusCode::FORBIDDEN)
                .fail("space is read-only"));
        }

        Ok(WritableSpace(space))
    }
}
