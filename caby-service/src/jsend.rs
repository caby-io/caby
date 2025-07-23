use std::default;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

const STATUS_SUCCESS: &'static str = "success";
const STATUS_FAIL: &'static str = "fail";
const STATUS_ERROR: &'static str = "error";

const GENERIC_INTERNAL_SERVER_ERROR: &'static str = "internal server error";

#[derive(Serialize)]
pub struct MessageJSend<T: Serialize> {
    pub status: &'static str,
    pub message: T,
}

#[derive(Serialize)]
pub struct DataJSend<T: Serialize> {
    pub status: &'static str,
    pub data: T,
}

// Variants
#[derive(Default, Clone)]
pub struct Unknown;
#[derive(Clone)]
pub struct Success<T: Serialize>(T);
#[derive(Default, Clone)]
pub struct Fail<T: Serialize>(T);
#[derive(Clone)]
pub struct Error(String);

#[derive(Default)]
pub struct JSendBuilder<U> {
    // TODO: consider using hyper and Into
    status_code: StatusCode,
    jsend_type: U,
}

impl JSendBuilder<Unknown> {
    pub fn new() -> Self {
        JSendBuilder {
            status_code: StatusCode::NOT_IMPLEMENTED,
            ..Default::default()
        }
    }
}

impl<U> JSendBuilder<U> {
    pub fn status_code(mut self, status_code: StatusCode) -> JSendBuilder<U> {
        self.status_code = status_code;
        self
    }

    pub fn success<T: Serialize>(mut self, data: T) -> JSendBuilder<Success<T>> {
        let mut status_code = StatusCode::OK;
        if self.status_code != StatusCode::NOT_IMPLEMENTED {
            status_code = self.status_code;
        }
        JSendBuilder {
            status_code,
            jsend_type: Success(data),
        }
    }

    pub fn fail<T: Serialize>(mut self, data: T) -> JSendBuilder<Fail<T>> {
        let mut status_code = StatusCode::BAD_REQUEST;
        if self.status_code != StatusCode::NOT_IMPLEMENTED {
            status_code = self.status_code;
        }
        JSendBuilder {
            status_code,
            jsend_type: Fail(data),
        }
    }

    pub fn error(mut self, message: impl Into<String>) -> JSendBuilder<Error> {
        let mut status_code = StatusCode::INTERNAL_SERVER_ERROR;
        if self.status_code != StatusCode::NOT_IMPLEMENTED {
            status_code = self.status_code;
        }
        JSendBuilder {
            status_code,
            jsend_type: Error(message.into()),
        }
    }

    pub fn internal_error(mut self) -> JSendBuilder<Error> {
        JSendBuilder {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            jsend_type: Error(GENERIC_INTERNAL_SERVER_ERROR.to_string()),
        }
    }
}

impl<T: Serialize> IntoResponse for JSendBuilder<Success<T>> {
    fn into_response(self) -> Response {
        (
            self.status_code,
            serde_json::to_string_pretty(&DataJSend {
                status: STATUS_SUCCESS,
                data: self.jsend_type.0,
            })
            .unwrap(),
        )
            .into_response()
    }
}

impl<T: Serialize> IntoResponse for JSendBuilder<Fail<T>> {
    fn into_response(self) -> Response {
        (
            self.status_code,
            serde_json::to_string_pretty(&DataJSend {
                status: STATUS_FAIL,
                data: self.jsend_type.0,
            })
            .unwrap(),
        )
            .into_response()
    }
}

impl IntoResponse for JSendBuilder<Error> {
    fn into_response(self) -> Response {
        (
            self.status_code,
            serde_json::to_string_pretty(&MessageJSend {
                status: STATUS_ERROR,
                message: self.jsend_type.0,
            })
            .unwrap(),
        )
            .into_response()
    }
}
