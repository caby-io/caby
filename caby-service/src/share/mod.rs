use std::{
    collections::{BTreeSet, HashMap},
    path::Path,
};

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jiff::Timestamp;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::{
    files::{has_ext, CABY_SHARE_SPEC_EXT},
    user::Permission,
    Result,
};

mod share_access;
pub mod share_download_token;
mod share_password;
mod share_spec;
mod share_store;

pub use share_download_token as download_token;
pub use share_spec::{spec_root, ShareSpec, SpecAuth, SpecFlow};
pub use share_store::{
    cleanup_spec, get_shares_in_space, load_state, reconcile_spec, remove_state,
};

pub(crate) use share_password::hash_format;

pub const CABY_SHARE_STATE_FILE: &str = "share.json";
const SHARE_DEFAULT_FILTER: &[fn(&str) -> bool] = &[is_share_spec];

fn is_share_spec(name: &str) -> bool {
    has_ext(Path::new(name), CABY_SHARE_SPEC_EXT)
}

// todo: support files + dirs
pub fn is_filtered(name: &str) -> bool {
    SHARE_DEFAULT_FILTER.iter().any(|rule| rule(name))
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShareAuth {
    Open,
    Password { hash: String },
}

// todo: implement this
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareLimits {
    pub max_file_bytes: Option<u64>,
    pub max_bytes_per_day: Option<u64>,
    pub max_files_per_day: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ShareAccessFlow {
    pub auth: ShareAuth,
    pub permissions: BTreeSet<Permission>,
    pub limits: Option<ShareLimits>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Grant {
    pub permissions: BTreeSet<Permission>,
    pub created_at: Timestamp,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Share {
    pub id: String,
    pub owner_id: String,
    pub space: String,
    pub spec_path: String,
    pub root_entry: String,

    pub account_allowlist: HashMap<String, Grant>,
    pub guest_allowlist: HashMap<String, Grant>,

    pub account_flows: Vec<ShareAccessFlow>,
    pub guest_flows: Vec<ShareAccessFlow>,

    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

impl TryFrom<SpecFlow> for ShareAccessFlow {
    type Error = crate::Error;

    fn try_from(flow: SpecFlow) -> Result<Self> {
        let auth = match flow.auth {
            SpecAuth::Open => ShareAuth::Open,
            SpecAuth::Password(plaintext) => ShareAuth::password(&plaintext)?,
            SpecAuth::Hash(hash) => {
                if hash_format(&hash).is_none() {
                    return Err(anyhow!(
                        "unsupported password hash format (use argon2 '$argon2id$', bcrypt '$2b$', or sha512-crypt '$6$')"
                    ));
                }
                ShareAuth::Password { hash }
            }
        };
        Ok(ShareAccessFlow {
            auth,
            permissions: flow.permissions,
            limits: flow.limits,
        })
    }
}

impl Share {
    pub fn new(
        owner_id: &str,
        space: &str,
        spec_path: &str,
        root_entry: &str,
        account_flows: Vec<ShareAccessFlow>,
        guest_flows: Vec<ShareAccessFlow>,
        expires_at: Option<Timestamp>,
    ) -> Self {
        let mut id_bytes = [0u8; 32];
        rand::rng().fill(&mut id_bytes);

        Self {
            id: URL_SAFE_NO_PAD.encode(id_bytes),
            owner_id: owner_id.to_owned(),
            space: space.to_owned(),
            spec_path: spec_path.to_owned(),
            root_entry: root_entry.to_owned(),
            account_allowlist: HashMap::new(),
            guest_allowlist: HashMap::new(),
            account_flows,
            guest_flows,
            created_at: Timestamp::now(),
            expires_at,
        }
    }

    pub fn from_spec(
        space: &str,
        spec_path: &Path,
        spec: ShareSpec,
        existing: Option<Share>,
        actor: Option<&str>,
    ) -> Result<Self> {
        let root = spec_root(spec_path)
            .ok_or_else(|| anyhow!("not a share spec path: {:?}", spec_path))?;
        let root_entry = root.to_string_lossy().into_owned();
        let spec_path = spec_path.to_string_lossy().into_owned();

        let account_flows = spec
            .account_flows
            .into_iter()
            .map(ShareAccessFlow::try_from)
            .collect::<Result<Vec<_>>>()?;
        let guest_flows = spec
            .guest_flows
            .into_iter()
            .map(ShareAccessFlow::try_from)
            .collect::<Result<Vec<_>>>()?;

        let share = match existing {
            Some(prev) => Share {
                id: prev.id,
                owner_id: prev.owner_id,
                space: space.to_owned(),
                spec_path,
                root_entry,
                account_allowlist: prev.account_allowlist,
                guest_allowlist: prev.guest_allowlist,
                account_flows,
                guest_flows,
                created_at: prev.created_at,
                expires_at: spec.expires_at,
            },
            None => Share::new(
                actor.unwrap_or_default(),
                space,
                &spec_path,
                &root_entry,
                account_flows,
                guest_flows,
                spec.expires_at,
            ),
        };

        Ok(share)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(account_flows: Vec<ShareAccessFlow>, guest_flows: Vec<ShareAccessFlow>) -> Share {
        Share::new(
            "holden",
            "rocinante",
            "photos/public.share.caby",
            "photos",
            account_flows,
            guest_flows,
            None,
        )
    }

    fn spec(guest_perms: &[Permission]) -> ShareSpec {
        ShareSpec {
            account_flows: vec![],
            guest_flows: vec![SpecFlow {
                auth: SpecAuth::Open,
                permissions: guest_perms.iter().copied().collect(),
                limits: None,
            }],
            expires_at: None,
        }
    }

    #[test]
    fn new_generates_unique_non_empty_ids() {
        let a = sample(vec![], vec![]);
        let b = sample(vec![], vec![]);
        assert!(!a.id.is_empty());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn hides_share_specs_from_shares() {
        assert!(is_filtered("photos.share.caby"));
        assert!(is_filtered("a.b.share.caby"));
        assert!(!is_filtered("photo.jpg"));
        assert!(!is_filtered("notes.txt"));
        assert!(!is_filtered("share.caby"));
    }

    #[test]
    fn from_spec_mints_an_id_then_carries_it_across_edits() {
        let spec_path = std::path::Path::new("photos/public.share.caby");
        let first = Share::from_spec(
            "rocinante",
            spec_path,
            spec(&[Permission::View]),
            None,
            Some("holden"),
        )
        .unwrap();
        assert!(!first.id.is_empty());
        assert_eq!(first.owner_id, "holden");
        assert_eq!(first.root_entry, "photos");
        assert_eq!(first.spec_path, "photos/public.share.caby");
        assert!(first.can_any_guest(Permission::View));

        let second = Share::from_spec(
            "rocinante",
            spec_path,
            spec(&[Permission::View, Permission::Download]),
            Some(first.clone()),
            Some("naomi"),
        )
        .unwrap();

        assert_eq!(second.id, first.id);
        assert_eq!(second.created_at, first.created_at);
        assert_eq!(
            second.owner_id, "holden",
            "owner is carried, not reassigned"
        );
        assert!(second.can_any_guest(Permission::Download));
    }
}
