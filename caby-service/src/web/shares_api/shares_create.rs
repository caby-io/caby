use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jiff::Timestamp;
use path_clean::PathClean;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

use crate::{
    auth::AuthUser,
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    jsend::JSendBuilder,
    share::{hash_format, Share, ShareLimits, ShareSpec, SpecAuth, SpecFlow},
    space::{Space, SpaceDir},
    user::{try_hash_password, Permission},
    Result,
};

#[derive(Deserialize)]
pub struct CreateShareRequest {
    #[serde(default)]
    pub dir: String,
    pub name: String,
    #[serde(default)]
    pub account_flows: Vec<CreateFlow>,
    #[serde(default)]
    pub guest_flows: Vec<CreateFlow>,
    pub expires_at: Option<Timestamp>,
}

#[derive(Deserialize)]
pub struct CreateFlow {
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub permissions: BTreeSet<Permission>,
    pub limits: Option<ShareLimits>,
}

#[derive(Serialize)]
struct CreateShareResponse {
    id: String,
    space: String,
    root_entry: String,
    created_at: Timestamp,
    expires_at: Option<Timestamp>,
}

impl TryFrom<CreateFlow> for SpecFlow {
    type Error = crate::Error;

    fn try_from(flow: CreateFlow) -> Result<Self> {
        let auth = match (flow.password, flow.password_hash) {
            (Some(plaintext), _) => SpecAuth::Hash(try_hash_password(&plaintext)?),
            (None, Some(hash)) => SpecAuth::Hash(hash),
            (None, None) => SpecAuth::Open,
        };
        Ok(SpecFlow {
            auth,
            permissions: flow.permissions,
            limits: flow.limits,
        })
    }
}

pub async fn handle_create_share(
    space: Space,
    auth: AuthUser,
    Json(req): Json<CreateShareRequest>,
) -> Response {
    let resp = JSendBuilder::new();

    let Some(account) = auth.as_account() else {
        return resp
            .status_code(StatusCode::FORBIDDEN)
            .fail("only account users can create shares")
            .into_response();
    };

    if let Some(expires_at) = req.expires_at {
        if expires_at <= Timestamp::now() {
            return resp.fail("must not expire in the past").into_response();
        }
    }

    for flow in req.account_flows.iter().chain(req.guest_flows.iter()) {
        if flow.permissions.is_empty() {
            return resp
                .fail("each flow must grant at least one permission")
                .into_response();
        }
        if flow.password.is_some() && flow.password_hash.is_some() {
            return resp
                .fail("a flow must not set both password and password_hash")
                .into_response();
        }
        if let Some(hash) = &flow.password_hash {
            if hash_format(hash).is_none() {
                return resp
                    .fail("password_hash must be argon2 '$argon2id$', bcrypt '$2b$', or sha512-crypt '$6$'")
                    .into_response();
            }
        }
    }

    if req.name.is_empty()
        || req.name.contains('/')
        || req.name.contains('\\')
        || has_ext(Path::new(&req.name), CABY_SHARE_SPEC_EXT)
    {
        return resp
            .fail("share name must be a single filename without a path or the .share.caby suffix")
            .into_response();
    }

    let dir = PathBuf::from(&req.dir).clean();
    let live_dir = match space.join(SpaceDir::LIVE, &dir) {
        Ok(path) => path,
        Err(err) => {
            return resp
                .fail(format!("invalid share directory: {}", err))
                .into_response();
        }
    };
    match fs::metadata(&live_dir).await {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => {
            return resp
                .fail("share target must be a directory")
                .into_response()
        }
        Err(_) => {
            return resp
                .fail("share target directory does not exist")
                .into_response()
        }
    }

    let spec_path = dir.join(format!("{}.share.caby", req.name)).clean();
    let spec_live = match space.join(SpaceDir::LIVE, &spec_path) {
        Ok(path) => path,
        Err(err) => {
            return resp
                .fail(format!("invalid share path: {}", err))
                .into_response();
        }
    };
    if fs::try_exists(&spec_live).await.unwrap_or(false) {
        return resp
            .status_code(StatusCode::CONFLICT)
            .fail("a share with that name already exists here")
            .into_response();
    }

    let (account_flows, guest_flows) = match (
        req.account_flows
            .into_iter()
            .map(SpecFlow::try_from)
            .collect::<Result<Vec<_>>>(),
        req.guest_flows
            .into_iter()
            .map(SpecFlow::try_from)
            .collect::<Result<Vec<_>>>(),
    ) {
        (Ok(accounts), Ok(guests)) => (accounts, guests),
        _ => {
            warn!("could not hash share flow password");
            return resp.internal_error().into_response();
        }
    };

    let spec = ShareSpec {
        account_flows,
        guest_flows,
        expires_at: req.expires_at,
    };

    let yaml = match String::try_from(&spec) {
        Ok(yaml) => yaml,
        Err(err) => {
            warn!("could not emit share spec {:?}: {:#}", spec_path, err);
            return resp.internal_error().into_response();
        }
    };
    if let Err(err) = fs::write(&spec_live, yaml).await {
        warn!("could not write share spec {:?}: {:#}", spec_live, err);
        return resp.internal_error().into_response();
    }

    let mut share = match Share::from_spec(&space.name, &spec_path, spec, None) {
        Ok(share) => share,
        Err(err) => {
            warn!("could not build share from spec {:?}: {:#}", spec_path, err);
            return resp.internal_error().into_response();
        }
    };
    share.owner_id = account.name.clone();

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
