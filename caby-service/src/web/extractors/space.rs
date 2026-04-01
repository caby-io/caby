use axum::extract::{Path, RawPathParams};
use std::{any, sync::Arc};

use crate::{
    config::Config,
    jsend::{Fail, JSendBuilder},
    space::Space,
    web::files_api::files_list::{FilesPathParams, FILE_NOT_FOUND},
};
use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::IntoResponse,
    Extension, RequestPartsExt,
};
use serde::Serialize;

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

        let Some(space_config) = cfg.spaces.get(&path_params.space) else {
            // todo: log specific err
            return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        };

        Ok(space_config.clone().into())
    }
}
