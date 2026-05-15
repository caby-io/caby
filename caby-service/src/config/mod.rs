use crate::{
    config::{
        auth::AuthConfig,
        config_file::{get_config_path, ConfigFile},
        validate_config::is_valid_meta_filename,
    },
    space::Space,
    user::{SpaceAccess, User},
    Result,
};
use anyhow::{anyhow, Context};
use arc_swap::ArcSwap;
use chacha20poly1305::{aead::OsRng, ChaCha20Poly1305, Key, KeyInit};
use serde::Deserialize;
use std::{collections::HashMap, env::var, path::PathBuf, sync::Arc};

pub mod auth;
mod config_file;
mod validate_config;

#[derive(Clone, Deserialize)]
pub struct SpaceConfig {
    pub name: String,
    pub display: String,
    pub path: PathBuf,
}

impl From<&SpaceConfig> for Space {
    fn from(val: &SpaceConfig) -> Self {
        Space {
            name: val.name.clone(),
            display: val.display.clone(),
            path: val.path.clone(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct UserSpaceConfig {
    name: String,
    permissions: Vec<String>,
}

impl From<&UserSpaceConfig> for SpaceAccess {
    fn from(val: &UserSpaceConfig) -> Self {
        SpaceAccess {
            name: val.name.clone(),
            permissions: val.permissions.clone(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub path: PathBuf,
    pub email: Option<String>,
    pub activation_token: Option<String>,
    pub spaces: Vec<UserSpaceConfig>,
}

impl From<&UserConfig> for User {
    fn from(val: &UserConfig) -> Self {
        User {
            name: val.name.clone(),
            path: val.path.clone(),
            email: val.email.clone(),
            activation_token: val.activation_token.clone(),
            space_access: val.spaces.iter().map(|s| s.into()).collect(),
        }
    }
}

// config that can be hot reloaded
#[derive(Clone)]
pub struct Runtime {
    pub spaces: HashMap<String, SpaceConfig>,
    pub users: HashMap<String, UserConfig>,
    // todo: roles
}

#[derive(Clone)]
pub struct Config {
    // system settings
    pub directory_meta_filename: String,
    pub home_path: PathBuf,
    pub users_path: PathBuf,
    pub spaces_path: PathBuf,

    // secrets
    pub upload_token_key: Key,

    // application settings
    pub auth: AuthConfig,
    pub runtime: Arc<ArcSwap<Runtime>>,
}

impl Config {
    pub fn find_user(&self, name: &str) -> Option<UserConfig> {
        self.runtime.load().users.get(name).cloned()
    }

    pub async fn new() -> Result<Self> {
        let mut builder = ConfigBuilder::new();
        let home_path = var("CABY_HOME_PATH").context("missing CABY_HOME_PATH")?;

        // Load minimal settings from env
        builder
            .try_set_dir_meta_filename(var("CABY_DIRECTORY_META_FILENAME").ok())?
            .try_set_home_path(Some(home_path))?
            .try_set_users_path(var("CABY_USERS_PATH").ok())?
            .try_set_spaces_path(var("CABY_SPACES_PATH").ok())?;

        // Load from config
        let config_file = ConfigFile::new_from_path(get_config_path()?).await?;

        let auth = config_file
            .auth
            .map(AuthConfig::try_from)
            .transpose()?
            .unwrap_or_default();
        builder.try_set_auth(Some(auth))?;

        let Some(spaces_path) = builder.spaces_path.clone() else {
            return Err(anyhow!("no valid spaces path from environment variables"));
        };
        builder.try_set_spaces(Some(
            config_file
                .spaces
                .into_iter()
                .map(|s| s.into_space_config(&spaces_path))
                .collect(),
        ))?;

        let Some(users_path) = builder.users_path.clone() else {
            return Err(anyhow!("no valid users path from environment variables"));
        };
        builder.try_set_users(Some(
            config_file
                .users
                .into_iter()
                .map(|u| u.into_user_config(&users_path))
                .collect(),
        ));

        // Load overrides from env
        // todo: load overrides from env

        builder.build()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    directory_meta_filename: Option<String>,
    home_path: Option<PathBuf>,
    users_path: Option<PathBuf>,
    spaces_path: Option<PathBuf>,
    auth: Option<AuthConfig>,
    spaces: HashMap<String, SpaceConfig>,
    users: HashMap<String, UserConfig>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    // overrides the field if the value is set
    fn try_override<T>(field: &mut Option<T>, value: Option<T>) {
        if value.is_none() {
            return;
        }

        *field = value
    }

    pub fn try_set_dir_meta_filename(&mut self, filename: Option<String>) -> Result<&mut Self> {
        let Some(f) = filename else { return Ok(self) };
        is_valid_meta_filename(&f)?;
        self.directory_meta_filename = Some(f);
        Ok(self)
    }

    pub fn try_set_home_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        let pb = p.into();
        self.home_path = Some(pb.clone());
        self.try_set_users_path(Some(pb.join("users")))?;
        self.try_set_spaces_path(Some(pb.join("spaces")))?;
        Ok(self)
    }

    pub fn try_set_users_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        self.users_path = Some(p.into());
        Ok(self)
    }

    pub fn try_set_spaces_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        self.spaces_path = Some(p.into());
        Ok(self)
    }

    pub fn try_set_auth(&mut self, auth: Option<AuthConfig>) -> Result<&mut Self> {
        let Some(a) = auth else {
            return Ok(self);
        };
        self.auth = Some(a);
        Ok(self)
    }

    pub fn try_set_spaces(&mut self, spaces: Option<Vec<SpaceConfig>>) -> Result<&mut Self> {
        let Some(sv) = spaces else {
            return Ok(self);
        };
        for s in sv {
            self.spaces.insert(s.name.clone(), s);
        }
        Ok(self)
    }

    pub fn try_set_users(&mut self, users: Option<Vec<UserConfig>>) -> Result<&mut Self> {
        let Some(uv) = users else {
            return Ok(self);
        };
        uv.iter().for_each(|u| {
            self.users.insert(u.name.to_lowercase(), u.clone());
        });
        Ok(self)
    }

    pub fn build(self) -> Result<Config> {
        let runtime = Runtime {
            spaces: self.spaces,
            users: self.users,
        };

        Ok(Config {
            directory_meta_filename: self.directory_meta_filename.ok_or(anyhow!(
                "missing directory meta filename (CABY_DIRECTORY_META_FILENAME)"
            ))?,
            // todo: get from file
            upload_token_key: ChaCha20Poly1305::generate_key(&mut OsRng),
            home_path: self.home_path.ok_or(anyhow!("missing home path"))?,
            users_path: self.users_path.ok_or(anyhow!("missing users path"))?,
            spaces_path: self.spaces_path.ok_or(anyhow!("missing spaces path"))?,
            auth: self.auth.ok_or(anyhow!("missing auth config"))?,
            runtime: Arc::new(ArcSwap::from_pointee(runtime)),
        })
    }
}
