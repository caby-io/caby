use std::{collections::BTreeSet, path::Path as StdPath};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    jsend::JSendBuilder,
    share::{Share, ShareAuth, ShareFlow},
    space::Space,
    user::Permission,
};

#[derive(Deserialize)]
pub struct ShareIdParam {
    pub id: String,
}

#[derive(Serialize, Default)]
struct GateInfo {
    available: bool,
    requires_password: bool,
    permissions: BTreeSet<Permission>,
}

impl From<&ShareFlow> for GateInfo {
    fn from(flow: &ShareFlow) -> Self {
        GateInfo {
            available: true,
            requires_password: matches!(flow.auth, ShareAuth::Password { .. }),
            permissions: flow.permissions.clone(),
        }
    }
}

#[derive(Serialize)]
struct GetShareResponse {
    id: String,
    space: String,
    root_name: String,
    expired: bool,
    member: GateInfo,
    guest: GateInfo,
}

pub async fn handle_get_share(space: Space, Path(params): Path<ShareIdParam>) -> Response {
    let share = match Share::load(&space, &params.id).await {
        Ok(Some(share)) => share,
        Ok(None) => {
            return JSendBuilder::new()
                .status_code(StatusCode::NOT_FOUND)
                .fail("share not found")
                .into_response()
        }
        Err(err) => {
            warn!("could not load share {}: {:#}", params.id, err);
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let expired = share.is_expired();
    let member = share
        .member_flow
        .as_ref()
        .map(GateInfo::from)
        .unwrap_or_default();
    let guest = share
        .guest_flow
        .as_ref()
        .map(GateInfo::from)
        .unwrap_or_default();
    let root_name = StdPath::new(&share.root_entry)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| share.root_entry.clone());

    JSendBuilder::new()
        .success(GetShareResponse {
            id: share.id,
            space: share.space,
            root_name,
            expired,
            member,
            guest,
        })
        .into_response()
}
