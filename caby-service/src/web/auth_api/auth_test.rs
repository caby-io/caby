use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};

use crate::{config::Config, jsend::JSendBuilder};

pub async fn handle_test_auth(State(cfg): State<Config>) -> Response {
    JSendBuilder::new().success("success").into_response()
}
