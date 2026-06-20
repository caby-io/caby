#![allow(unused)]

use anyhow::Context;
use std::time::Duration;

use crate::housekeeping::housekeeping;

pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Request},
    http::Method,
    Router, ServiceExt,
};
use config::Config;
use tokio::{net::TcpListener, task, time};
use tower::Layer;
use tower_http::{cors::CorsLayer, normalize_path::NormalizePathLayer, trace::TraceLayer};
use tracing::info;

mod auth;
mod bootstrap;
mod config;
mod ctx;
mod download;
mod error;
mod files;
mod housekeeping;
mod img_thumbs;
mod jsend;
mod space;
mod state;
mod upload;
mod user;
mod validation;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let cfg = Config::new().await?;
    // Create early to pay slow init early
    let _vips = bootstrap::vips::init()?;
    bootstrap::fs::init(&cfg).await?;

    // housekeeping
    let handle = task::spawn({
        let cfg = cfg.clone();
        async move {
            let mut interval = time::interval(Duration::from_secs(60 * 30));
            loop {
                interval.tick().await;
                housekeeping(&cfg).await;
            }
        }
    });

    let cors_layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers(web::headers::cors_allowed_request_headers())
        .allow_origin(cfg.urls.cors_allowed_origins.clone())
        .allow_credentials(true);

    let state = state::AppState::new(cfg).await?;

    let app = Router::new()
        .nest("/v0", web::api_router(&state))
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
        .with_state(state);
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .context("could not bind listener on 0.0.0.0:8080")?;
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .context("server crashed")?;

    Ok(())
}

async fn handler_files(files_path: Option<Path<String>>) {
    let path = files_path.map_or("/".to_string(), |Path(p)| p);
    println!("{:?}", path)
}
