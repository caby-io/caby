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
        let config = Config::from_ref(state);

        // todo: find in config

        let resp = JSendBuilder::new().fail(FILE_NOT_FOUND);

        return Err(resp);

        // todo: test
        Ok(Space {
            name: "test".to_string(),
            path: Path::new(&config.spaces_path).join("test"),
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
