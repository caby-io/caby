use axum::{
    routing::{get, post},
    Router,
};

use crate::config::Config;

mod files_api;

pub fn api_router() -> Router<Config> {
    Router::new().nest(
        "/files",
        Router::new()
            // Wildcards don't support the base path so this is required
            .route("/list/", get(files_api::handle_list_files))
            .route("/list/*file_path", get(files_api::handle_list_files))
            .route(
                "/download/*file_path",
                get(files_api::handle_download_files),
            )
            .route("/upload", post(files_api::handle_register_upload))
            .route("/delete", post(files_api::handle_delete_files))
            .route("/move", post(files_api::handle_move_files)),
    )
}
