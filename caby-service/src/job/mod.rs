use std::{path::PathBuf, time::Duration};

use jiff::Timestamp;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Input {
    // Shares
    ScanShares {
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
}

impl Input {
    pub fn locks(&self) -> Vec<LockKey> {
        match self {
            Self::ScanShares { space } => vec![LockKey::Space(space.clone())],
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
        }
    }

    pub fn timeout(&self) -> Duration {
        match self {
            Self::ScanShares { .. } => Duration::from_secs(600),
            Self::ReconcileShare { .. } => Duration::from_secs(60),
            Self::MoveShare { .. } => Duration::from_secs(60),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LockKey {
    Instance,
    Space(String),
    Path { space: String, path: PathBuf },
}

impl LockKey {
    pub fn conflicts(&self, other: &LockKey) -> bool {
        use LockKey::*;
        match (self, other) {
            (Instance, _) | (_, Instance) => true,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Priority {
    Interactive,
    Background,
}

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub input: Input,
    pub status: Status,
    pub priority: Priority,
    pub created_at: Timestamp,
    pub attempts: u32,
    pub not_before: Option<Timestamp>,
}

impl Job {
    pub fn new(input: Input, priority: Priority) -> Self {
        Self {
            id: xid::new().to_string(),
            input,
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
