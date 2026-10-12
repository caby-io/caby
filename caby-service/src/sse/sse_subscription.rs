use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::files::{has_ext, CABY_SHARE_SPEC_EXT};

use super::sse_event::{EntryChange, SseEvent};

#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionKind {
    Dir,
    Tree,
    Shares,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Subscription {
    Dir { space: String, path: PathBuf },
    Tree { space: String },
    Shares { space: String },
}

fn change_paths(change: &EntryChange) -> impl Iterator<Item = &Path> {
    std::iter::once(change.path.as_str())
        .chain(change.from.as_deref())
        .map(Path::new)
}

impl Subscription {
    pub fn kind(&self) -> SubscriptionKind {
        match self {
            Subscription::Dir { .. } => SubscriptionKind::Dir,
            Subscription::Tree { .. } => SubscriptionKind::Tree,
            Subscription::Shares { .. } => SubscriptionKind::Shares,
        }
    }

    pub fn matches(&self, event: &SseEvent) -> bool {
        let Some(change) = event.entry_change() else {
            return false;
        };

        match self {
            Subscription::Tree { space } => change.space == *space,
            Subscription::Dir { space, path } => {
                change.space == *space
                    && change_paths(change).any(|p| p.parent() == Some(path.as_path()))
            }
            Subscription::Shares { space } => {
                change.space == *space
                    && change_paths(change).any(|p| has_ext(p, CABY_SHARE_SPEC_EXT))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sse::sse_event::SsePayload;

    fn moved(space: &str, path: &str, from: &str) -> SseEvent {
        SseEvent {
            payload: SsePayload::EntryMoved(EntryChange {
                space: space.to_owned(),
                path: path.to_owned(),
                from: Some(from.to_owned()),
                entry: None,
            }),
            actor: None,
        }
    }

    fn created(space: &str, path: &str) -> SseEvent {
        SseEvent {
            payload: SsePayload::EntryCreated(EntryChange {
                space: space.to_owned(),
                path: path.to_owned(),
                from: None,
                entry: None,
            }),
            actor: None,
        }
    }

    fn dir(space: &str, path: &str) -> Subscription {
        Subscription::Dir {
            space: space.to_owned(),
            path: PathBuf::from(path),
        }
    }

    #[test]
    fn dir_matches_direct_children_only() {
        let sub = dir("rocinante", "docs");
        assert!(sub.matches(&created("rocinante", "docs/a.txt")));
        assert!(!sub.matches(&created("rocinante", "docs/nested/a.txt")));
        assert!(!sub.matches(&created("rocinante", "docs")));
    }

    #[test]
    fn dir_matches_move_source_and_destination() {
        let sub = dir("rocinante", "docs");
        assert!(sub.matches(&moved("rocinante", "photos/a.txt", "docs/a.txt")));
        assert!(sub.matches(&moved("rocinante", "docs/a.txt", "photos/a.txt")));
        assert!(!sub.matches(&moved("rocinante", "music/a.txt", "photos/a.txt")));
    }

    #[test]
    fn dir_root_matches_top_level() {
        let sub = dir("rocinante", "");
        assert!(sub.matches(&created("rocinante", "a.txt")));
        assert!(!sub.matches(&created("rocinante", "docs/a.txt")));
    }

    #[test]
    fn other_space_never_matches() {
        let event = created("tachi", "docs/a.share.caby");
        assert!(!dir("rocinante", "docs").matches(&event));
        assert!(!Subscription::Tree {
            space: "rocinante".to_owned()
        }
        .matches(&event));
        assert!(!Subscription::Shares {
            space: "rocinante".to_owned()
        }
        .matches(&event));
    }

    #[test]
    fn tree_matches_any_depth() {
        let sub = Subscription::Tree {
            space: "rocinante".to_owned(),
        };
        assert!(sub.matches(&created("rocinante", "docs/nested/a.txt")));
    }

    #[test]
    fn shares_matches_spec_files_only() {
        let sub = Subscription::Shares {
            space: "rocinante".to_owned(),
        };
        assert!(sub.matches(&created("rocinante", "docs/holiday.share.caby")));
        assert!(sub.matches(&moved("rocinante", "docs/a.txt", "docs/a.share.caby")));
        assert!(!sub.matches(&created("rocinante", "docs/a.txt")));
    }

    #[test]
    fn stream_events_never_match() {
        let sub = Subscription::Tree {
            space: "rocinante".to_owned(),
        };
        assert!(!sub.matches(&SseEvent::reset()));
        assert!(!sub.matches(&SseEvent::connected("k3J")));
    }
}
