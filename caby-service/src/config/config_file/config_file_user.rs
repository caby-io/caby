use std::path::Path;

use crate::config::{UserConfig, UserSpaceConfig};

pub struct SpaceAccess {
    pub name: String,
    pub permissions: Vec<String>,
}

impl Into<UserSpaceConfig> for SpaceAccess {
    fn into(self) -> UserSpaceConfig {
        UserSpaceConfig {
            name: self.name,
            permissions: self.permissions,
        }
    }
}

pub struct ConfigFileUser {
    pub name: String,
    pub email: Option<String>,
    pub spaces: Vec<SpaceAccess>,
}

impl ConfigFileUser {
    pub fn into_user_config(self, users_path: &Path) -> UserConfig {
        return UserConfig {
            name: self.name.clone(),
            path: users_path.join(self.name),
            email: self.email.clone(),
            spaces: self.spaces.into_iter().map(|s| s.into()).collect(),
        };
    }
}
