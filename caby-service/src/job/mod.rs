use std::{ffi::OsString, path::PathBuf, time::Duration};

use jiff::Timestamp;

pub const THUMB_TIMEOUT: Duration = Duration::from_secs(60);
pub const THUMBS_BATCH_SIZE: u32 = 200;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Input {
    // Shares
    ScanSpace {
        space: String,
    },
    ReconcileShare {
        space: String,
        path: PathBuf,
    },
    MoveShare {
        space: String,
        from: PathBuf,
        to: PathBuf,
    },

    // Thumbnails
    GenerateFileThumb {
        space: String,
        path: PathBuf,
    },
    GenerateDirThumbs {
        space: String,
        dir: PathBuf,
        recursive: bool,
        after: Option<OsString>,
    },
}

impl Input {
    pub fn locks(&self) -> Vec<LockKey> {
        match self {
            Self::ScanSpace { space } => vec![LockKey::Space(space.clone())],
            Self::ReconcileShare { space, path } => vec![LockKey::Path {
                space: space.clone(),
                path: path.clone(),
            }],
            Self::MoveShare { space, from, to } => vec![
                LockKey::Path {
                    space: space.clone(),
                    path: from.clone(),
                },
                LockKey::Path {
                    space: space.clone(),
                    path: to.clone(),
                },
            ],
            Self::GenerateFileThumb { .. } | Self::GenerateDirThumbs { .. } => {
                vec![LockKey::Thumbnails]
            }
        }
    }

    pub fn timeout(&self) -> Duration {
        match self {
            Self::ScanSpace { .. } => Duration::from_secs(600),
            Self::ReconcileShare { .. } => Duration::from_secs(60),
            Self::MoveShare { .. } => Duration::from_secs(60),
            Self::GenerateFileThumb { .. } => THUMB_TIMEOUT,
            Self::GenerateDirThumbs { .. } => THUMB_TIMEOUT * THUMBS_BATCH_SIZE,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LockKey {
    Instance,
    Space(String),
    Path { space: String, path: PathBuf },
    Thumbnails,
}

impl LockKey {
    pub fn conflicts(&self, other: &LockKey) -> bool {
        use LockKey::*;
        match (self, other) {
            (Instance, _) | (_, Instance) => true,
            (Thumbnails, Thumbnails) => true,
            (Thumbnails, _) | (_, Thumbnails) => false,
            (Space(a), Space(b)) => a == b,
            (Space(s), Path { space, .. }) | (Path { space, .. }, Space(s)) => s == space,
            (Path { space: a, path: p }, Path { space: b, path: q }) => {
                a == b && (p.starts_with(q) || q.starts_with(p))
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Pending,
    Running,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Priority {
    Interactive,
    Background,
}

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub input: Input,
    pub actor: Option<String>,
    pub status: Status,
    pub priority: Priority,
    pub created_at: Timestamp,
    pub attempts: u32,
    pub not_before: Option<Timestamp>,
}

impl Job {
    pub fn new(input: Input, priority: Priority, actor: Option<String>) -> Self {
        Self {
            id: xid::new().to_string(),
            input,
            actor,
            status: Status::Pending,
            priority,
            created_at: Timestamp::now(),
            attempts: 0,
            not_before: None,
        }
    }

    pub fn is_ready(&self, now: Timestamp) -> bool {
        self.not_before.is_none_or(|at| at <= now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(space: &str, path: &str) -> LockKey {
        LockKey::Path {
            space: space.to_owned(),
            path: PathBuf::from(path),
        }
    }

    #[test]
    fn thumbnail_locks_serialize_each_other() {
        assert!(LockKey::Thumbnails.conflicts(&LockKey::Thumbnails));
    }

    #[test]
    fn thumbnail_locks_ignore_space_and_path_locks() {
        let space = LockKey::Space("rocinante".to_owned());
        let path = path("rocinante", "holiday");
        assert!(!LockKey::Thumbnails.conflicts(&space));
        assert!(!LockKey::Thumbnails.conflicts(&path));
        assert!(!space.conflicts(&LockKey::Thumbnails));
        assert!(!path.conflicts(&LockKey::Thumbnails));
    }

    #[test]
    fn thumbnail_locks_yield_to_instance() {
        assert!(LockKey::Thumbnails.conflicts(&LockKey::Instance));
        assert!(LockKey::Instance.conflicts(&LockKey::Thumbnails));
    }
}
