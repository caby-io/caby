use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::warn;

use crate::{
    auth::AuthUser,
    jsend::JSendBuilder,
    share::{list_in_space, Share},
    space::Space,
};

#[derive(Serialize)]
struct ShareSummary {
    id: String,
    root_entry: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    expired: bool,
}

impl From<&Share> for ShareSummary {
    fn from(share: &Share) -> Self {
        ShareSummary {
            id: share.id.clone(),
            root_entry: share.root_entry.clone(),
            created_at: share.created_at,
            expires_at: share.expires_at,
            expired: share.is_expired(),
        }
    }
}

#[derive(Serialize)]
struct ShareListResponse {
    shares: Vec<ShareSummary>,
}

pub async fn handle_list_shares(space: Space, auth: AuthUser) -> Response {
    let resp = JSendBuilder::new();

    let Some(account) = auth.as_account() else {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("only accounts can list shares")
            .into_response();
    };

    let shares = match list_in_space(&space).await {
        Ok(shares) => shares,
        Err(err) => {
            warn!("could not list shares in {}: {:#}", space.name, err);
            return resp.internal_error().into_response();
        }
    };

    let shares: Vec<ShareSummary> = shares
        .iter()
        .filter(|share| share.owner_id == account.name)
        .map(ShareSummary::from)
        .collect();

    resp.success(ShareListResponse { shares }).into_response()
}
