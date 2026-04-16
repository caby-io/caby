use axum::{
    middleware,
    routing::{get, patch, post, put},
    Router,
};

use crate::{
    auth::{self, auth_middleware},
    config::Config,
};

mod auth_api;
mod extractors;
mod files_api;
mod headers;
mod spaces_api;
mod upload;
mod users_api;

pub fn api_router(cfg: &Config) -> Router<Config> {
    Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/token/lookup", post(auth_api::handle_token_lookup))
                .route(
                    "/token/activate",
                    post(auth_api::handle_user_token_activation),
                )
                .route("/login", post(auth_api::handle_login))
                .route("/logout", post(auth_api::handle_logout)), // .route("/test", get(auth_api::handle_test_auth)),
        )
        // .nest(
        //     "/users",
        // )
        .nest(
            "/spaces",
            Router::new()
                .route("/", get(spaces_api::handle_list_spaces))
                .route("/{space}", get(spaces_api::handle_show_space)),
        )
        .nest(
            "/files",
            Router::new()
                // Wildcards don't support the base path so this is required
                .route("/list/{space}", get(files_api::handle_list_files))
                .route(
                    "/list/{space}/{*file_path}",
                    get(files_api::handle_list_files),
                )
                .route("/overview/{space}", get(files_api::handle_files_overview))
                .route(
                    "/overview/{space}/{*file_path}",
                    get(files_api::handle_files_overview),
                )
                .route(
                    "/download/{space}/{*file_path}",
                    get(files_api::handle_download_files),
                )
                .route("/{space}", put(files_api::handle_put_files))
                .route("/{space}/{*file_path}", put(files_api::handle_put_files))
                .route("/upload/{space}", post(files_api::handle_register_upload))
                .route(
                    "/upload/{space}/chunk/{id}/{*file_path}",
                    put(files_api::handle_upload_chunk),
                )
                .route(
                    "/upload/{space}/{id}/{*file_path}",
                    patch(files_api::handle_update_upload),
                )
                .route(
                    "/upload/{space}/{id}",
                    post(files_api::handle_complete_upload),
                )
                // .route("/upload/complete", post(files_api::handle_complete_upload))
                .route("/delete/{space}", post(files_api::handle_delete_files))
                .route("/move/{space}", post(files_api::handle_move_files)),
        )
}
