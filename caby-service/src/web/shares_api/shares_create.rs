use std::{collections::BTreeSet, path::PathBuf};

use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::AuthorizedUser,
    config::Config,
    jsend::JSendBuilder,
    share::{Share, ShareAccess, ShareAuth, ShareLimits, SharePermission},
    space::{Space, SpaceDir},
    Result,
};

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub root_entry: String,
    pub member_access: Option<CreateLane>,
    pub public_access: Option<CreateLane>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateLane {
    pub password: Option<String>,
    pub permissions: BTreeSet<SharePermission>,
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

// Converts into ShareAccess and hashes password if inputted
impl TryFrom<CreateLane> for ShareAccess {
    type Error = crate::Error;

    fn try_from(lane: CreateLane) -> Result<Self> {
        let auth = match lane.password {
            Some(ref plaintext) => ShareAuth::password(plaintext)?,
            None => ShareAuth::Open,
        };
        Ok(ShareAccess {
            auth,
            permissions: lane.permissions,
            limits: lane.limits,
        })
    }
}

pub async fn handle_create_share(
    State(cfg): State<Config>,
    space: Space,
    user: AuthorizedUser,
    Json(req): Json<CreateShareRequest>,
) -> Response {
    if let Some(expires_at) = req.expires_at {
        if expires_at <= Utc::now() {
            return JSendBuilder::new()
                .fail("must not expire in the past")
                .into_response();
        }
    }

    for lane in [req.member_access.as_ref(), req.public_access.as_ref()]
        .into_iter()
        .flatten()
    {
        if lane.permissions.is_empty() {
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

    let (member_access, public_access) = match (
        req.member_access.map(ShareAccess::try_from).transpose(),
        req.public_access.map(ShareAccess::try_from).transpose(),
    ) {
        (Ok(member), Ok(public)) => (member, public),
        _ => {
            warn!("could not hash share lane password");
            return JSendBuilder::new().internal_error().into_response();
        }
    };

    let share = Share::new(
        &user.user.name,
        &space.name,
        &root_entry,
        member_access,
        public_access,
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
