use std::time::Duration;

use axum::Router;

pub mod routes_files;

pub fn api_routes() -> Router {
    routes_files::routes()
}
