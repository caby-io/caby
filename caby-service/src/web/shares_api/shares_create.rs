use std::{collections::BTreeSet, path::PathBuf};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::AuthUser,
    config::Config,
    jsend::JSendBuilder,
    share::{Share, ShareAccessFlow, ShareAuth, ShareLimits},
    space::{Space, SpaceDir},
    user::Permission,
    Result,
};

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub root_entry: String,
    pub all_accounts: Option<CreateFlow>,
    pub all_guests: Option<CreateFlow>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateFlow {
    pub password: Option<String>,
    pub permissions: BTreeSet<Permission>,
    pub limits: Option<ShareLimits>,
}

#[derive(Serialize)]
struct CreateShareResponse {
    id: String,
    space: String,
    root_entry: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

impl TryFrom<CreateFlow> for ShareAccessFlow {
    type Error = crate::Error;

    fn try_from(flow: CreateFlow) -> Result<Self> {
        let auth = match flow.password {
            Some(ref plaintext) => ShareAuth::password(plaintext)?,
            None => ShareAuth::Open,
        };
        Ok(ShareAccessFlow {
            auth,
            permissions: flow.permissions,
            limits: flow.limits,
        })
    }
}

pub async fn handle_create_share(
    State(cfg): State<Config>,
    space: Space,
    user: AuthUser,
    Json(req): Json<CreateShareRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let Some(account) = user.as_account() else {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("only account users can create shares")
            .into_response();
    };

    if let Some(expires_at) = req.expires_at {
        if expires_at <= Utc::now() {
            return resp.fail("must not expire in the past").into_response();
        }
    }

    for flow in [req.all_accounts.as_ref(), req.all_guests.as_ref()]
        .into_iter()
        .flatten()
    {
        if flow.permissions.is_empty() {
            return resp
                .fail("each audience must grant at least one permission")
                .into_response();
        }
    }

    let cleaned_root = PathBuf::from(&req.root_entry).clean();
    let live_root = match space.join(SpaceDir::LIVE, &cleaned_root) {
        Ok(path) => path,
        Err(err) => {
            return resp
                .fail(format!("invalid root entry: {}", err))
                .into_response();
        }
    };
    match fs::try_exists(&live_root).await {
        Ok(true) => {}
        Ok(false) => {
            return resp.fail("root entry does not exist").into_response();
        }
        Err(err) => {
            warn!("could not check root entry {:?}: {:#}", live_root, err);
            return resp.internal_error().into_response();
        }
    }
    let root_entry = cleaned_root.to_string_lossy().into_owned();

    let (account_flows, guest_flows) = match (
        req.all_accounts.map(ShareAccessFlow::try_from).transpose(),
        req.all_guests.map(ShareAccessFlow::try_from).transpose(),
    ) {
        (Ok(accounts), Ok(guests)) => (
            accounts.into_iter().collect::<Vec<_>>(),
            guests.into_iter().collect::<Vec<_>>(),
        ),
        _ => {
            warn!("could not hash share flow password");
            return resp.internal_error().into_response();
        }
    };

    let share = Share::new(
        &account.name,
        &space.name,
        &root_entry,
        account_flows,
        guest_flows,
        req.expires_at,
    );
    if let Err(err) = share.save(&space).await {
        warn!("could not save share {}: {:#}", share.id, err);
        return resp.internal_error().into_response();
    }

    resp.success(CreateShareResponse {
        id: share.id,
        space: share.space,
        root_entry: share.root_entry,
        created_at: share.created_at,
        expires_at: share.expires_at,
    })
    .into_response()
}
