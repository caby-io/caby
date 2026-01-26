use std::sync::Arc;

// use crate::{Result};
use crate::error::RequestError;
use anyhow::anyhow;
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Clone, Debug)]
pub struct Ctx {}

impl Ctx {
    pub fn new() -> Self {
        Self {}
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = RequestError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, RequestError> {
        parts
            .extensions
            .get::<Result<Ctx, RequestError>>()
            .ok_or(anyhow!("missing context"))?
            .clone()
    }
}
