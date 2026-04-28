use crate::{
    auth::AuthorizedUser, config::Config, error::Result, jsend::JSendBuilder, space::Space,
    web::files_api::files_list::FilesPathParams,
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
    space: Space,
    user: AuthorizedUser,
    path_params: Path<FilesPathParams>,
    Json(payload): Json<PutEntryRequest>,
) -> Response {
    let resp = JSendBuilder::new();
    let rel_path = path_params
        .file_path
        .clone()
        .map_or(PathBuf::from(""), PathBuf::from);
    // let Ok(path) = space.join(&rel_path) else {
    //     return resp.fail("invalid path").into_response();
    // };

    let path = space.live().join(&rel_path);

    // todo: validate path is valid

    match payload.entry_type {
        PutEntryType::Directory => match create_dirs(&path, vec![payload.name]).await {
            Ok(_) => resp
                .status_code(StatusCode::CREATED)
                .success("dir created")
                .into_response(),
            Err(err) => {
                error!("could not create dir at {:?}: {:#}", path, err);
                resp.fail("could not create directory").into_response()
            }
        },
        _ => resp.fail("unhandled entry_type").into_response(),
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
