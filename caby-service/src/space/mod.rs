use anyhow::{anyhow, Ok};
use path_clean::PathClean;
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{config::SpaceConfig, Result};

#[derive(Serialize, Clone, Debug)]
pub struct Space {
    pub name: String,
    pub display: String,
    pub path: PathBuf,
}

pub enum SpaceDir {
    LIVE,
    META,
    SHARES,
    UPLOADS,
}

impl Space {
    pub fn join(&self, dir: SpaceDir, path: &Path) -> Result<PathBuf> {
        // todo: check if is valid?
        let cleaned_path = path.clean();
        let parent_path = match dir {
            SpaceDir::LIVE => self.live(),
            SpaceDir::META => self.meta(),
            SpaceDir::SHARES => self.shares(),
            SpaceDir::UPLOADS => self.uploads(),
        };
        let joined_path = parent_path.join(cleaned_path).clean();
        if !joined_path.starts_with(parent_path) {
            return Err(anyhow!("final path out of bounds of parent path"));
        };

        Ok(joined_path)
    }

    pub fn live(&self) -> PathBuf {
        self.path.join("live")
    }

    pub fn meta(&self) -> PathBuf {
        self.path.join("meta")
    }

    pub fn shares(&self) -> PathBuf {
        self.path.join("shares")
    }

    pub fn uploads(&self) -> PathBuf {
        self.path.join("uploads")
    }
}

pub fn select_space(
    spaces: &HashMap<String, SpaceConfig>,
    space_name: String,
) -> Option<SpaceConfig> {
    None
}
