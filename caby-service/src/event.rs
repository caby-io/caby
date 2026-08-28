use std::path::{Path, PathBuf};

use tokio::sync::mpsc;
use tracing::warn;

pub const BUFFER_SIZE: usize = 1024;

pub type Sender = mpsc::Sender<Event>;
pub type Receiver = mpsc::Receiver<Event>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EventKind {
    FileCreated,
    FileModified,
    FileRemoved,
    FileMoved { from: PathBuf },
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub space: String,
    pub path: PathBuf,
}

impl Event {
    fn new(space: impl Into<String>, path: impl Into<PathBuf>, kind: EventKind) -> Self {
        Self {
            space: space.into(),
            path: path.into(),
            kind,
        }
    }

    pub fn from_create(space: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::new(space, path, EventKind::FileCreated)
    }

    pub fn from_modify(space: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::new(space, path, EventKind::FileModified)
    }

    pub fn from_remove(space: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::new(space, path, EventKind::FileRemoved)
    }

    pub fn from_move(
        space: impl Into<String>,
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>,
    ) -> Self {
        Self::new(space, to, EventKind::FileMoved { from: from.into() })
    }

    pub fn paths(&self) -> Vec<&Path> {
        match &self.kind {
            EventKind::FileMoved { from } => vec![&self.path, from],
            _ => vec![&self.path],
        }
    }
}

pub fn channel() -> (Sender, Receiver) {
    mpsc::channel(BUFFER_SIZE)
}

pub fn emit(sender: &Sender, event: Event) {
    if let Err(err) = sender.try_send(event) {
        let event = err.into_inner();
        warn!(
            "dropped file event for {}/{}",
            event.space,
            event.path.display()
        );
    }
}
