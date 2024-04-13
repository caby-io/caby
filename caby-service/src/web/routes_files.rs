use crate::files::get_files;
use crate::{ctx::Ctx, jsend};
use crate::error::Result;
use axum::response::{IntoResponse, Response};
use axum::{extract::Path, routing::get, Extension, Json, Router};
use serde_json::{json, Value};

pub fn routes() -> Router {
    Router::new()
        .route("/files/", get(api_files))
        .route("/files/:file_path", get(api_files))
}

async fn api_files(ctx: Result<Ctx>, files_path: Option<Path<String>>) -> Response {
    let path = files_path.map_or("/".to_string(), |Path(p)| p);
    // println!("{:?}", path);

    let res = get_files(std::path::Path::new("/")).await;
    // println!("{:?}", res);

    jsend::JSendBuilder::new()
        .success("hello world").into_response()

    // Ok(body)
}
