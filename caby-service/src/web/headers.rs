use axum::{
    http::{header::ToStrError, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::warn;

use crate::jsend::JSendBuilder;

// The header key (&str) should always be a &'static str so we should not copy it into a String
pub enum HeaderError<'a> {
    MissingHeaderError(&'a str),
    HeaderParseError(&'a str),
}

impl<'a> IntoResponse for HeaderError<'a> {
    fn into_response(self) -> Response {
        return match self {
            HeaderError::MissingHeaderError(key) => JSendBuilder::new()
                .status_code(StatusCode::BAD_REQUEST)
                .fail(format!("missing '{}' header", key))
                .into_response(),
            HeaderError::HeaderParseError(key) => JSendBuilder::new()
                .status_code(StatusCode::BAD_REQUEST)
                .fail(format!("could not parse '{}' header as a string", key))
                .into_response(),
        };
    }
}

pub fn get_required_header<'a, 'b>(
    headers: &'a HeaderMap,
    key: &'b str,
) -> Result<&'a str, HeaderError<'b>> {
    headers
        .get(key)
        .ok_or(HeaderError::MissingHeaderError(key))?
        .to_str()
        .map_err(|e| HeaderError::HeaderParseError(key))
}

pub fn get_header<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    headers.get(key)?.to_str().ok()
}
