use path_clean::PathClean;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{config::SpaceConfig, Result};

#[derive(Clone, Debug)]
pub struct Space {
    pub name: String,
    pub path: PathBuf,
}

impl Space {
    pub fn join(&self, path: &Path) -> Result<PathBuf> {
        // todo: check if is valid
        Ok(self.path.join(path.clean()).clean())
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
