use path_clean::PathClean;
use std::{collections::HashMap, path::PathBuf};

use crate::{config::SpaceConfig, Result};

#[derive(Clone, Debug)]
pub struct Space {
    pub name: String,
    pub path: PathBuf,
}

impl Space {
    pub fn join(&self, path: &PathBuf) -> Result<PathBuf> {
        // todo: check if is valid
        Ok(self.path.join(path.clean()).clean())
    }
}

pub fn select_space(
    spaces: &HashMap<String, SpaceConfig>,
    space_name: String,
) -> Option<SpaceConfig> {
    None
}
