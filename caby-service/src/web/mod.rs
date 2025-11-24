use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::config::Config;

mod files_api;
mod headers;
mod upload;

pub fn api_router() -> Router<Config> {
    Router::new().nest(
        "/files",
        Router::new()
            // Wildcards don't support the base path so this is required
            .route("/list", get(files_api::handle_list_files))
            .route("/list/{*file_path}", get(files_api::handle_list_files))
            .route("/overview", get(files_api::handle_files_overview))
            .route(
                "/overview/{*file_path}",
                get(files_api::handle_files_overview),
            )
            .route(
                "/download/{*file_path}",
                get(files_api::handle_download_files),
            )
            .route("/", put(files_api::handle_put_files))
            .route("/{*file_path}", put(files_api::handle_put_files))
            .route("/upload", post(files_api::handle_register_upload))
            .route(
                "/upload/chunk/{id}/{*file}",
                put(files_api::handle_chunk_upload),
            )
            .route("/upload/{id}/{*file}", patch(files_api::handle_update_file))
            .route("/upload/{id}", post(files_api::handle_complete_upload))
            // .route("/upload/complete", post(files_api::handle_complete_upload))
            .route("/delete", post(files_api::handle_delete_files))
            .route("/move", post(files_api::handle_move_files)),
    )
}
