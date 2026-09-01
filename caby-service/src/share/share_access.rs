use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use jiff::Timestamp;
use path_clean::PathClean;

use crate::{
    auth::User,
    guest::Guest,
    space::{Space, SpaceDir},
    user::{Account, Permission},
    Result,
};

use super::{Grant, Share, ShareAccessFlow, ShareAuth};

impl ShareAccessFlow {
    pub fn grants(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn open_grants(&self, permission: Permission) -> bool {
        matches!(self.auth, ShareAuth::Open) && self.grants(permission)
    }
}

impl Share {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(at) => Timestamp::now() > at,
            None => false,
        }
    }

    pub fn can_account(&self, account: &Account, permission: Permission) -> bool {
        if let Some(grant) = self.account_allowlist.get(&account.name) {
            if grant.permissions.contains(&permission) {
                return true;
            }
        }

        // check if there's open access for accounts
        self.account_flows
            .iter()
            .any(|flow| flow.open_grants(permission))
            || self.can_any_guest(permission)
    }

    pub fn can_any_guest(&self, permission: Permission) -> bool {
        self.guest_flows
            .iter()
            .any(|flow| flow.open_grants(permission))
    }

    pub fn can_guest(&self, guest: &Guest, permission: Permission) -> bool {
        if let Some(grant) = self.guest_allowlist.get(&guest.id) {
            if grant.permissions.contains(&permission) {
                return true;
            }
        }

        self.can_any_guest(permission)
    }

    pub fn grant(&mut self, user: &User, permissions: BTreeSet<Permission>) {
        let (allowlist, id) = match user {
            User::Account(account) => (&mut self.account_allowlist, account.name.clone()),
            User::Guest(guest) => (&mut self.guest_allowlist, guest.id.clone()),
        };
        allowlist.insert(
            id,
            Grant {
                permissions,
                created_at: Timestamp::now(),
            },
        );
    }

    pub fn scope_path(&self, space: &Space, rel: &Path) -> Result<PathBuf> {
        let root = PathBuf::from(&self.root_entry).clean();
        let scoped = root.join(rel.clean()).clean();

        let live = space.join(SpaceDir::LIVE, &scoped)?;
        let root_live = space.live().join(&root);
        if !live.starts_with(&root_live) {
            return Err(anyhow!("path escapes share root"));
        }

        Ok(scoped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::SignedDuration;

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

    fn open_flow(perms: &[Permission]) -> ShareAccessFlow {
        ShareAccessFlow {
            auth: ShareAuth::Open,
            permissions: perms.iter().copied().collect(),
            limits: None,
        }
    }

    fn password_flow(perms: &[Permission]) -> ShareAccessFlow {
        ShareAccessFlow {
            auth: ShareAuth::Password {
                hash: "hash".to_owned(),
            },
            permissions: perms.iter().copied().collect(),
            limits: None,
        }
    }

    fn account(name: &str) -> Account {
        Account {
            name: name.to_owned(),
            path: PathBuf::from("/tmp"),
            email: None,
            activation_token: None,
            space_access: vec![],
        }
    }

    fn grant(perms: &[Permission]) -> Grant {
        Grant {
            permissions: perms.iter().copied().collect(),
            created_at: Timestamp::now(),
        }
    }

    #[test]
    fn is_expired_reflects_expiry() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.is_expired());
        share.expires_at = Some(Timestamp::now() - SignedDuration::from_mins(1));
        assert!(share.is_expired());
        share.expires_at = Some(Timestamp::now() + SignedDuration::from_mins(1));
        assert!(!share.is_expired());
    }

    #[test]
    fn open_grants_requires_open_auth_and_permission() {
        assert!(open_flow(&[Permission::View]).open_grants(Permission::View));
        assert!(!open_flow(&[Permission::View]).open_grants(Permission::Download));
        assert!(!password_flow(&[Permission::View]).open_grants(Permission::View));
    }

    #[test]
    fn can_any_guest_only_from_open_guest_flows() {
        let open = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(open.can_any_guest(Permission::View));
        assert!(!open.can_any_guest(Permission::Download));

        let locked = sample(vec![], vec![password_flow(&[Permission::View])]);
        assert!(!locked.can_any_guest(Permission::View));
    }

    #[test]
    fn can_guest_via_open_flow() {
        let share = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(share.can_guest(&Guest::new(), Permission::View));
        assert!(!share.can_guest(&Guest::new(), Permission::Download));
    }

    #[test]
    fn can_guest_via_allowlist() {
        let guest = Guest::new();
        let mut share = sample(vec![], vec![]);
        assert!(!share.can_guest(&guest, Permission::Download));

        share
            .guest_allowlist
            .insert(guest.id.clone(), grant(&[Permission::Download]));
        assert!(share.can_guest(&guest, Permission::Download));
        assert!(!share.can_guest(&guest, Permission::Delete));
        assert!(!share.can_guest(&Guest::new(), Permission::Download));
    }

    #[test]
    fn can_account_via_open_account_flow() {
        let share = sample(vec![open_flow(&[Permission::Write])], vec![]);
        assert!(share.can_account(&account("naomi"), Permission::Write));
        assert!(!share.can_account(&account("naomi"), Permission::Delete));
    }

    #[test]
    fn can_account_falls_through_to_open_guest_access() {
        let share = sample(vec![], vec![open_flow(&[Permission::View])]);
        assert!(share.can_account(&account("naomi"), Permission::View));
    }

    #[test]
    fn can_account_via_allowlist() {
        let mut share = sample(vec![], vec![]);
        assert!(!share.can_account(&account("naomi"), Permission::Delete));

        share
            .account_allowlist
            .insert("naomi".to_owned(), grant(&[Permission::Delete]));
        assert!(share.can_account(&account("naomi"), Permission::Delete));
        assert!(!share.can_account(&account("marco"), Permission::Delete));
    }

    #[test]
    fn owner_has_no_implicit_content_access() {
        let share = sample(vec![], vec![]);
        assert!(!share.can_account(&account("holden"), Permission::Delete));
    }

    #[test]
    fn grant_routes_by_principal_kind() {
        let mut share = sample(vec![], vec![]);

        let guest = Guest::new();
        let guest_id = guest.id.clone();
        share.grant(&User::Guest(guest), BTreeSet::from([Permission::View]));
        assert!(share.guest_allowlist.contains_key(&guest_id));
        assert!(share.account_allowlist.is_empty());

        share.grant(
            &User::Account(account("holden")),
            BTreeSet::from([Permission::Delete]),
        );
        assert!(share.account_allowlist.contains_key("holden"));
    }
}
