use crate::{auth::AuthorizedUser, config::Config, files, jsend, space::Space};
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct DeleteEntriesRequest {
    pub entries: Vec<String>,
    pub force: bool,
}

#[derive(Serialize)]
struct DeleteEntriesResponse {
    pub deleted: Vec<String>,
    pub errors: Vec<String>,
}

// todo: this should be archiving instead of deleting
pub async fn handle_delete_files(
    State(cfg): State<Config>,
    space: Space,
    user: AuthorizedUser,
    Json(req): Json<DeleteEntriesRequest>,
) -> Response {
    let mut deleted = vec![];
    let mut errors = vec![];

    for relative_path in req.entries {
        let rel_path = PathBuf::from(relative_path.clone()).clean();

        if let Err(err) = files::ops::remove(&space, &rel_path).await {
            errors.push(format!("{:?}: {:#}", relative_path, err));
            continue;
        }

        deleted.push(rel_path.to_str().unwrap().to_owned());
    }

    jsend::JSendBuilder::new()
        .success(DeleteEntriesResponse { deleted, errors })
        .into_response()
}
