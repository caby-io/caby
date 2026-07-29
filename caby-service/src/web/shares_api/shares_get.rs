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
    share::{Share, ShareAccessFlow, ShareAuth},
    space::Space,
    user::Permission,
};

#[derive(Deserialize)]
pub struct ShareIdParam {
    pub id: String,
}

#[derive(Serialize)]
struct FlowInfo {
    requires_password: bool,
    permissions: BTreeSet<Permission>,
}

impl From<&ShareAccessFlow> for FlowInfo {
    fn from(flow: &ShareAccessFlow) -> Self {
        FlowInfo {
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
    account_flows: Vec<FlowInfo>,
    guest_flows: Vec<FlowInfo>,
}

pub async fn handle_get_share(space: Space, Path(params): Path<ShareIdParam>) -> Response {
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

    let expired = share.is_expired();
    let account_flows: Vec<FlowInfo> = share.account_flows.iter().map(FlowInfo::from).collect();
    let guest_flows: Vec<FlowInfo> = share.guest_flows.iter().map(FlowInfo::from).collect();
    let root_name = StdPath::new(&share.root_entry)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| share.root_entry.clone());

    resp.success(GetShareResponse {
        id: share.id,
        space: share.space,
        root_name,
        expired,
        account_flows,
        guest_flows,
    })
    .into_response()
}
