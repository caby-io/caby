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
    share::{Share, ShareAuth, ShareFlow, ShareLimits},
    space::{Space, SpaceDir},
    user::Permission,
    Result,
};

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub root_entry: String,
    pub member_flow: Option<CreateFlow>,
    pub guest_flow: Option<CreateFlow>,
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

impl TryFrom<CreateFlow> for ShareFlow {
    type Error = crate::Error;

    fn try_from(flow: CreateFlow) -> Result<Self> {
        let auth = match flow.password {
            Some(ref plaintext) => ShareAuth::password(plaintext)?,
            None => ShareAuth::Open,
        };
        Ok(ShareFlow {
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
    let Some(account) = user.as_account() else {
        return JSendBuilder::new()
            .status_code(StatusCode::FORBIDDEN)
            .fail("only account users can create shares")
            .into_response();
    };

    if let Some(expires_at) = req.expires_at {
        if expires_at <= Utc::now() {
            return JSendBuilder::new()
                .fail("must not expire in the past")
                .into_response();
        }
    }

    for flow in [req.member_flow.as_ref(), req.guest_flow.as_ref()]
        .into_iter()
        .flatten()
    {
        if flow.permissions.is_empty() {
            return JSendBuilder::new()
                .fail("each audience must grant at least one permission")
                .into_response();
        }
    }

    let cleaned_root = PathBuf::from(&req.root_entry).clean();
    let live_root = match space.join(SpaceDir::LIVE, &cleaned_root) {
        Ok(path) => path,
        Err(err) => {
            return JSendBuilder::new()
                .fail(format!("invalid root entry: {}", err))
                .into_response();
        }
    };
    match fs::try_exists(&live_root).await {
        Ok(true) => {}
        Ok(false) => {
            return JSendBuilder::new()
                .fail("root entry does not exist")
                .into_response();
        }
        Err(err) => {
            warn!("could not check root entry {:?}: {:#}", live_root, err);
            return JSendBuilder::new().internal_error().into_response();
        }
    }
    let root_entry = cleaned_root.to_string_lossy().into_owned();

    let (member_flow, guest_flow) = match (
        req.member_flow.map(ShareFlow::try_from).transpose(),
        req.guest_flow.map(ShareFlow::try_from).transpose(),
    ) {
        (Ok(member), Ok(guest)) => (member, guest),
        _ => {
            warn!("could not hash share flow password");
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let share = Share::new(
        &account.name,
        &space.name,
        &root_entry,
        member_flow,
        guest_flow,
        req.expires_at,
    );
    if let Err(err) = share.save(&space).await {
        warn!("could not save share {}: {:#}", share.id, err);
        return JSendBuilder::new().internal_error().into_response();
    }

    JSendBuilder::new()
        .success(CreateShareResponse {
            id: share.id,
            space: share.space,
            root_entry: share.root_entry,
            created_at: share.created_at,
            expires_at: share.expires_at,
        })
        .into_response()
}
