use crate::{
    config::Config,
    ctx::Ctx,
    error::Result,
    files::joined_path,
    jsend::{self, JSendBuilder},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use path_clean::PathClean;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;
use tracing::error;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PutEntryType {
    Directory,
    File,
}

#[derive(Deserialize)]
pub struct PutEntryRequest {
    pub entry_type: PutEntryType,
    pub name: String,
    // todo: this probably isnt the best for raw files?
    pub content: Option<String>,
}

// used to create directories and small, inline, files
pub async fn handle_put_files(
    State(cfg): State<Config>,
    ctx: Result<Ctx>,
    files_path: Option<Path<String>>,
    Json(payload): Json<PutEntryRequest>,
) -> Response {
    let resp = JSendBuilder::new();
    let rel_path = files_path.map_or(PathBuf::from(""), |Path(p)| PathBuf::from(p));
    let Some(path) = joined_path(&cfg.live_path, &rel_path) else {
        return resp.fail("invalid path").into_response();
    };

    // todo: validate path is valid

    match payload.entry_type {
        PutEntryType::Directory => match create_dirs(&path, vec![payload.name]).await {
            Ok(_) => {
                return resp
                    .status_code(StatusCode::CREATED)
                    .success("dir created")
                    .into_response()
            }
            Err(err) => {
                error!("could not create dir: {}", err);
                return resp.fail("could not create directory").into_response();
            }
        },
        _ => return resp.fail("unhandled entry_type").into_response(),
    }
}

// todo: move these

// todo: validate parent dir exists
// todo: validate dir names are valid
async fn create_dirs(parent_path: &PathBuf, dirs: Vec<String>) -> Result<()> {
    for d in dirs {
        let dir_path = parent_path.join(d).clean();
        fs::create_dir(dir_path).await?;
    }
    Ok(())
}
