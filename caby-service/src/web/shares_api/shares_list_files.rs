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
    files::{build_entries, Entry, EntryFields},
    jsend::JSendBuilder,
    share::{download_token, is_filtered, Share},
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

fn build_download_url(cfg: &Config, space: &str, id: &str, path: &str, token: &str) -> String {
    let mut url = cfg.urls.backend.clone();
    url.set_path(&format!("/v0/shares/{}/{}/download/{}", space, id, path));
    url.query_pairs_mut().append_pair("token", token);
    url.to_string()
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

    entries.retain(|entry| !is_filtered(&entry.name));

    let can_download = authorize_share(auth.as_ref(), &share, Permission::Download);
    let root_entry = share.root_entry.as_str();
    for entry in &mut entries {
        let scoped_path = entry.path.clone();

        let display_path = match StdPath::new(&entry.path).strip_prefix(root_entry) {
            Ok(stripped) => stripped.to_string_lossy().into_owned(),
            Err(_) => entry.path.clone(),
        };
        entry.path = display_path.clone();

        if can_download {
            if let Some(EntryFields::File { download_url, .. }) = &mut entry.entry_fields {
                match download_token::generate_token(&cfg, &share.id, &scoped_path) {
                    Ok(token) => {
                        *download_url = Some(build_download_url(
                            &cfg,
                            &space.name,
                            &share.id,
                            &display_path,
                            &token,
                        ));
                    }
                    Err(err) => warn!(
                        "could not mint download token for {}: {:#}",
                        scoped_path, err
                    ),
                }
            }
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
