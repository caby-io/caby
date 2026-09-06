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

    pub live: PathBuf,
    pub meta: PathBuf,
    pub uploads: PathBuf,

    pub managed: bool,
    pub readonly: bool,
}

pub enum SpaceDir {
    LIVE,
    META,
    UPLOADS,
}

impl Space {
    pub fn join(&self, dir: SpaceDir, path: &Path) -> Result<PathBuf> {
        // todo: check if is valid?
        let cleaned_path = path.clean();
        let parent_path: &Path = match dir {
            SpaceDir::LIVE => &self.live,
            SpaceDir::META => &self.meta,
            SpaceDir::UPLOADS => &self.uploads,
        };
        let joined_path = parent_path.join(cleaned_path).clean();
        if !joined_path.starts_with(parent_path) {
            return Err(anyhow!("final path out of bounds of parent path"));
        };

        Ok(joined_path)
    }
}

pub fn select_space(
    spaces: &HashMap<String, SpaceConfig>,
    space_name: String,
) -> Option<SpaceConfig> {
    None
}
