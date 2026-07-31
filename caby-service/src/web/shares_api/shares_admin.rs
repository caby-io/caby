use std::collections::{BTreeMap, BTreeSet};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::warn;

use crate::{
    auth::AuthUser,
    jsend::JSendBuilder,
    share::{Grant, Share, ShareAccessFlow, ShareAuth, ShareLimits},
    space::Space,
    user::Permission,
    web::shares_api::ShareIdParam,
};

#[derive(Serialize)]
struct FlowDetail {
    requires_password: bool,
    permissions: BTreeSet<Permission>,
    limits: Option<ShareLimits>,
}

impl From<&ShareAccessFlow> for FlowDetail {
    fn from(flow: &ShareAccessFlow) -> Self {
        FlowDetail {
            requires_password: matches!(flow.auth, ShareAuth::Password { .. }),
            permissions: flow.permissions.clone(),
            limits: flow.limits.clone(),
        }
    }
}

// Almost 1:1 with share, except flows
#[derive(Serialize)]
struct AdminShareResponse {
    id: String,
    space: String,
    owner_id: String,
    root_entry: String,
    account_flows: Vec<FlowDetail>,
    guest_flows: Vec<FlowDetail>,
    account_allowlist: BTreeMap<String, Grant>,
    guest_allowlist: BTreeMap<String, Grant>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl From<&Share> for AdminShareResponse {
    fn from(share: &Share) -> Self {
        AdminShareResponse {
            id: share.id.clone(),
            space: share.space.clone(),
            owner_id: share.owner_id.clone(),
            root_entry: share.root_entry.clone(),
            account_flows: share.account_flows.iter().map(FlowDetail::from).collect(),
            guest_flows: share.guest_flows.iter().map(FlowDetail::from).collect(),
            account_allowlist: share
                .account_allowlist
                .iter()
                .map(|(id, grant)| (id.clone(), grant.clone()))
                .collect(),
            guest_allowlist: share
                .guest_allowlist
                .iter()
                .map(|(id, grant)| (id.clone(), grant.clone()))
                .collect(),
            created_at: share.created_at,
            expires_at: share.expires_at,
        }
    }
}

pub async fn handle_admin_get_share(
    space: Space,
    auth: AuthUser,
    Path(params): Path<ShareIdParam>,
) -> Response {
    let resp = JSendBuilder::new();

    let Some(account) = auth.as_account() else {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("only accounts can administer shares")
            .into_response();
    };

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

    if !share.can_admin(account) {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("not authorized to administer this share")
            .into_response();
    }

    resp.success(AdminShareResponse::from(&share))
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_detail_omits_password_hash() {
        let flow = ShareAccessFlow {
            auth: ShareAuth::Password {
                hash: "argon2-secret".to_owned(),
            },
            permissions: BTreeSet::from([Permission::View]),
            limits: None,
        };

        let detail = FlowDetail::from(&flow);
        assert!(detail.requires_password);

        let json = serde_json::to_string(&detail).unwrap();
        assert!(!json.contains("argon2-secret"));
        assert!(!json.contains("hash"));
    }
}
