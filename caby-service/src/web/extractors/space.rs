use axum::extract::{Path as ExtractPath, RawPathParams};
use std::{any, path::Path, sync::Arc};

// use crate::{Result};
use crate::{
    config::Config,
    error::RequestError,
    jsend::{Fail, JSendBuilder},
    space::Space,
};
use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::IntoResponse,
    Extension, RequestPartsExt,
};
use serde::Serialize;

// #[derive(Clone, Debug)]
// pub struct Ctx {}

// impl Ctx {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

const FILE_NOT_FOUND: &'static str = "file not found";

impl<S> FromRequestParts<S> for Space
where
    Config: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = JSendBuilder<Fail<&'static str>>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cfg = Config::from_ref(state);

        let Ok(path_params) = parts.extract::<RawPathParams>().await else {
            // todo: message err
            return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        };

        let Some((_, space)) = path_params.iter().find(|(k, _)| *k == "space") else {
            // todo: message warn
            return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        };

        // let Ok(space) = ExtractPath::<(String)>::from_request_parts(parts, &()).await else {
        //     return Err(JSendBuilder::new().fail(FILE_NOT_FOUND));
        // };

        // todo: test
        Ok(Space {
            name: "test".to_string(),
            path: Path::new(&cfg.spaces_path).join("test"),
        })
    }

    // async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, RequestError> {
    //     // let Extension(cfg) = parts
    //     //     .extract::<Extension<Config>>()
    //     //     .await
    //     //     .map_err(anyhow!("failed"))?;

    //     let Extension(cfg) = parts
    //         .extract_with_state::
    //         .get::<Result<Config, RequestError>>()
    //         .ok_or(anyhow!("missing context"))?
    //         .clone();

    //     Ok(Space)
    // }
}
