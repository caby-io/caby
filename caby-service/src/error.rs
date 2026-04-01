use axum::response::{IntoResponse, Response};
use std::sync::Arc;
use tracing::error;

use crate::jsend::JSendBuilder;

// pub type Result<T> = core::result::Result<T, Error>;
pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;

// Make our own error that wraps `anyhow::Error`.
#[derive(Clone, Debug)]
pub struct RequestError(Arc<anyhow::Error>);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        error!("unhandled response error: {}", self.0);
        JSendBuilder::new().error("server error").into_response()
    }
}

// impl From<Error> for RequestError {
//     fn from(value: Error) -> Self {
//         Self(Arc::new(value))
//     }
// }

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RequestError>`. That way you don't need to do that manually.
impl<E> From<E> for RequestError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(Arc::new(err.into()))
    }
}

// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         error!("unhandled error: {:?}", self);
//         JSendBuilder::new().error("server error").into_response()
//     }
// }

// impl core::fmt::Display for Error {
//     fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
//         write!(fmt, "{self:?}")
//     }
// }

// impl std::error::Error for Error {}

// #[derive(Clone, Debug)]
// pub enum Error {
//     CtxMissing,
//     Generic(String),
//     UploadTokenParseError,
//     IoError(Arc<io::Error>),
// }

// impl Error {
//     pub fn into_generic(reason: impl Into<String>, err: Arc<dyn std::error::Error>) -> Error {
//         return Self::Generic(format!("{}: {}", reason.into(), err));
//     }
// }

// impl From<io::Error> for Error {
//     fn from(value: io::Error) -> Self {
//         Error::IoError(Arc::new(value))
//     }
// }

// // todo: jsend this?
// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         error!("unhandled error: {:?}", self);
//         JSendBuilder::new().error("server error").into_response()
//     }
// }
