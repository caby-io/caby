use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{io, sync::Arc};
use tracing::error;

use crate::jsend::JSendBuilder;

pub type Result<T> = core::result::Result<T, Error>;

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug)]
pub enum Error {
    CtxMissing,
    Generic(String),
    UploadTokenParseError,
    IoError(Arc<io::Error>),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(Arc::new(value))
    }
}

// todo: jsend this?
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("unhandled error: {:?}", self);
        JSendBuilder::new().error("server error").into_response()
    }
}
