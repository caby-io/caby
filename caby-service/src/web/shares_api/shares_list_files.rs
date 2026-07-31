use std::path::{Path as StdPath, PathBuf};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    auth::{authorize_share, AuthUser},
    config::Config,
    files::{build_entries, Entry},
    jsend::JSendBuilder,
    share::Share,
    space::Space,
    user::Permission,
};

#[derive(Deserialize)]
pub struct ListParams {
    id: String,
    path: Option<String>,
}

#[derive(Serialize)]
struct ListShareResponse {
    path: String,
    parent_dir: Option<String>,
    entries: Vec<Entry>,
}

pub async fn handle_list_share_files(
    State(cfg): State<Config>,
    space: Space,
    auth: Option<AuthUser>,
    Path(params): Path<ListParams>,
) -> Response {
    let resp = JSendBuilder::new();

    let share = match Share::load(&space, &params.id).await {
        Ok(Some(share)) => share,
        Ok(None) => {
            return resp
                .status_code(StatusCode::NOT_FOUND)
                .fail("share not found")
                .into_response()
        }
        Err(err) => {
            warn!("could not load share {}: {:#}", params.id, err);
            return resp.internal_error().into_response();
        }
    };

    if share.is_expired() {
        return resp
            .status_code(StatusCode::GONE)
            .fail("share has expired")
            .into_response();
    }

    if !authorize_share(auth.as_ref(), &share, Permission::View) {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("not authorized for this share")
            .into_response();
    }

    let rel = params.path.map_or(PathBuf::from(""), PathBuf::from);
    let scoped = match share.scope_path(&space, &rel) {
        Ok(scoped) => scoped,
        Err(_) => return resp.fail("invalid path").into_response(),
    };

    let mut entries = match build_entries(&cfg, &space, &scoped).await {
        Ok(entries) => entries,
        Err(err) => {
            warn!(
                "could not list share {} at {:?}: {:#}",
                share.id, scoped, err
            );
            return resp.internal_error().into_response();
        }
    };

    let root_entry = share.root_entry.as_str();
    for entry in &mut entries {
        if let Ok(stripped) = StdPath::new(&entry.path).strip_prefix(root_entry) {
            entry.path = stripped.to_string_lossy().into_owned();
        }
    }

    let parent_dir = rel
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned());

    resp.success(ListShareResponse {
        path: rel.to_string_lossy().into_owned(),
        parent_dir,
        entries,
    })
    .into_response()
}
