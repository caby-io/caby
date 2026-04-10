use crate::{
    auth::AuthorizedUser,
    config::Config,
    jsend,
    space::{Space, SpaceDir},
};
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

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
        let Ok(path) = space.join(SpaceDir::LIVE, &rel_path) else {
            // todo
            errors.push(format!("{:?} invaild path", relative_path));
            continue;
        };

        let Ok(metadata) = fs::metadata(path.clone()).await else {
            // todo: make error structured and parseable
            errors.push(format!("{:?} not found", relative_path));
            continue;
        };

        if metadata.is_dir() {
            if let Err(err) = fs::remove_dir_all(path).await {
                errors.push(format!("couldn't delete {:?}: {:?}", relative_path, err));
                continue;
            }
            deleted.push(rel_path.to_str().unwrap().to_owned());
            continue;
        }

        if let Err(err) = fs::remove_file(path).await {
            errors.push(format!("couldn't delete {:?}: {:?}", relative_path, err));
            continue;
        }
        deleted.push(rel_path.to_str().unwrap().to_owned());
    }

    jsend::JSendBuilder::new()
        .success(DeleteEntriesResponse { deleted, errors })
        .into_response()
}
