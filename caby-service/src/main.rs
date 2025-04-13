// #![allow(unused)]

pub use self::error::{Error, Result};

use axum::{
    extract::Path,
    http::{header, Method},
    Router,
};
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

mod ctx;
mod error;
mod files;
mod jsend;
mod web;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // TEMP
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::OPTIONS, Method::POST, Method::DELETE])
        .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
        // allow requests from any origin
        // TODO make this come from an env var
        .allow_origin(Any);

    let app = Router::new()
        .nest("/v0", web::api_router())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(
                    tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO),
                )
                .on_response(
                    tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO),
                ),
            // .on_failure(tower_http::trace::DefaultOnFailure::new().level(tracing::Level::WARN)),
        )
        .layer(cors_layer);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler_files(files_path: Option<Path<String>>) {
    let path = files_path.map_or("/".to_string(), |Path(p)| p);
    println!("{:?}", path)
}
