use crate::Error;
use crate::Result;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

#[derive(Clone, Debug)]
pub struct Ctx {}

impl Ctx {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::CtxMissing)?
            .clone()
    }
}
