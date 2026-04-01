#![allow(unused)]

pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Request},
    Router, ServiceExt,
};
use config::Config;
use init::init;
use tokio::net::TcpListener;
use tower::Layer;
use tower_http::{
    cors::{Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    trace::TraceLayer,
};

mod auth;
mod config;
mod ctx;
mod error;
mod files;
mod init;
mod jsend;
mod space;
mod user;
mod web;

#[tokio::main]
async fn main() {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Build config
    let cfg = Config::new().await.expect("could not load config");
    init(&cfg).await.expect("init error");

    // Initialize paths
    // todo: log something when dir is created
    // fs::create_dir_all(&cfg.live_path).await.unwrap();
    // fs::create_dir_all(&cfg.meta_path).await.unwrap();
    // TEMP uploads housekeeping
    // fs::remove_dir_all(&cfg.uploads_path).await.unwrap();
    // fs::create_dir_all(&cfg.uploads_path).await.unwrap();

    // TEMP
    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        // temp
        .allow_headers(Any)
        // .allow_headers([header::ACCEPT, header::CONTENT_TYPE])
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
        .layer(cors_layer)
        .with_state(cfg);
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();
}

async fn handler_files(files_path: Option<Path<String>>) {
    let path = files_path.map_or("/".to_string(), |Path(p)| p);
    println!("{:?}", path)
}
