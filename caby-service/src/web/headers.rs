use axum::{
    http::{
        header::{HeaderName, ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
};

use crate::jsend::JSendBuilder;

pub static HEADER_CABY_UPLOAD_TOKEN: HeaderName = HeaderName::from_static("caby-upload-token");
pub static HEADER_CABY_CHUNK_INDEX: HeaderName = HeaderName::from_static("caby-chunk-index");
pub static HEADER_CABY_USER_NAME: HeaderName = HeaderName::from_static("caby-user-name");

pub fn cors_allowed_request_headers() -> Vec<HeaderName> {
    vec![
        ACCEPT,
        AUTHORIZATION,
        CONTENT_TYPE,
        HEADER_CABY_UPLOAD_TOKEN.clone(),
        HEADER_CABY_CHUNK_INDEX.clone(),
        HEADER_CABY_USER_NAME.clone(),
    ]
}

pub enum HeaderError {
    MissingHeaderError(&'static str),
    HeaderParseError(&'static str),
}

impl IntoResponse for HeaderError {
    fn into_response(self) -> Response {
        match self {
            HeaderError::MissingHeaderError(key) => JSendBuilder::new()
                .status_code(StatusCode::BAD_REQUEST)
                .fail(format!("missing '{}' header", key))
                .into_response(),
            HeaderError::HeaderParseError(key) => JSendBuilder::new()
                .status_code(StatusCode::BAD_REQUEST)
                .fail(format!("could not parse '{}' header as a string", key))
                .into_response(),
        }
    }
}

pub fn get_header<'a>(headers: &'a HeaderMap, key: &HeaderName) -> Option<&'a str> {
    headers.get(key)?.to_str().ok()
}

pub fn get_required_header<'a>(
    headers: &'a HeaderMap,
    key: &'static HeaderName,
) -> Result<&'a str, HeaderError> {
    headers
        .get(key)
        .ok_or(HeaderError::MissingHeaderError(key.as_str()))?
        .to_str()
        .map_err(|_| HeaderError::HeaderParseError(key.as_str()))
}
