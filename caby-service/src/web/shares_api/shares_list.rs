use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use jiff::Timestamp;
use serde::Serialize;
use tracing::warn;

use crate::{
    config::Config,
    jsend::JSendBuilder,
    share::{get_shares_in_space, Share},
    space::Space,
    web::extractors::RequireAccount,
};

#[derive(Serialize)]
struct ShareSummary {
    id: String,
    owner_id: String,
    root_entry: String,
    created_at: Timestamp,
    expires_at: Option<Timestamp>,
    expired: bool,
}

impl From<&Share> for ShareSummary {
    fn from(share: &Share) -> Self {
        ShareSummary {
            id: share.id.clone(),
            owner_id: share.owner_id.clone(),
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

pub async fn handle_list_shares(
    space: Space,
    _: RequireAccount,
    State(cfg): State<Config>,
) -> Response {
    let resp = JSendBuilder::new();

    let shares = match get_shares_in_space(&cfg.shares_path, &space).await {
        Ok(shares) => shares,
        Err(err) => {
            warn!("could not list shares in {}: {:#}", space.name, err);
            return resp.internal_error().into_response();
        }
    };

    let shares: Vec<ShareSummary> = shares.iter().map(ShareSummary::from).collect();

    resp.success(ShareListResponse { shares }).into_response()
}
