use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::warn;

pub type Result<T> = core::result::Result<T, Error>;

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

#[derive(Clone, Debug)]
pub enum Error {
    CtxMissing
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("unhandled error: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, "server error").into_response()
    }
}
