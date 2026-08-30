use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

#[derive(Default)]
pub struct PathLocks {
    locks: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
}

pub struct PathGuard(#[allow(dead_code)] OwnedMutexGuard<()>);

impl PathLocks {
    fn key(space: &str, path: &Path) -> String {
        format!("{}\0{}", space, path.display())
    }

    pub async fn acquire(&self, space: &str, path: &Path) -> PathGuard {
        let mutex = {
            let mut locks = self.locks.lock().unwrap_or_else(|err| err.into_inner());
            locks.entry(Self::key(space, path)).or_default().clone()
        };
        PathGuard(mutex.lock_owned().await)
    }

    pub async fn acquire_pair(
        &self,
        space: &str,
        first: &Path,
        second: &Path,
    ) -> (PathGuard, PathGuard) {
        if Self::key(space, first) <= Self::key(space, second) {
            let a = self.acquire(space, first).await;
            let b = self.acquire(space, second).await;
            (a, b)
        } else {
            let b = self.acquire(space, second).await;
            let a = self.acquire(space, first).await;
            (a, b)
        }
    }
}

#[cfg(test)]
impl PathGuard {
    pub async fn test() -> PathGuard {
        PathLocks::default()
            .acquire("test", Path::new("test"))
            .await
    }
}
