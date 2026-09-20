use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::{jsend, web::extractors::RequireAccount};

#[derive(Serialize)]
pub struct MeResponse<'a> {
    user: &'a str,
    email: Option<&'a str>,
}

pub async fn handle_me(RequireAccount(account): RequireAccount) -> Response {
    jsend::JSendBuilder::new()
        .success(MeResponse {
            user: &account.name,
            email: account.email.as_deref(),
        })
        .into_response()
}
