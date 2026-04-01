use std::path::PathBuf;

use anyhow::anyhow;
use tokio::fs::try_exists;

use crate::Result;

pub enum UserType {
    Human,
    Agent,
}

pub struct SpaceAccess {
    pub name: String,
    pub permissions: Vec<String>,
}

pub struct User {
    // config values
    pub name: String,
    pub path: PathBuf,
    pub email: Option<String>,
    pub activation_token: Option<String>,
    pub space_access: Vec<SpaceAccess>,
    // todo: profile
    // pub user_type: UserType,
}

impl User {
    // If the user's directory exists, it's activated
    pub async fn is_activated(&self) -> Result<bool> {
        let user_dir_exists = match try_exists(&self.path).await {
            Ok(e) => e,
            Err(err) => {
                return Err(anyhow!("could not find user dir: {}", err));
            }
        };

        return Ok(user_dir_exists);
    }
}
