use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

const VALID_API_KEY: &str = "Bearer my_secret_api_key";

async fn auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if auth_header == VALID_API_KEY {
        // If the API key matches, proceed to the next handler
        Ok(next.run(req).await)
    } else {
        // Otherwise, return Unauthorized
        Err(StatusCode::UNAUTHORIZED)
    }
}
