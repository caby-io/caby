use crate::{
    config::{
        config_file::{get_config_path, ConfigFile},
        validate_config::is_valid_meta_filename,
    },
    space::Space,
    Result,
};
use anyhow::{anyhow, Context, Error};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::var,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

pub mod config_file;
mod validate_config;

#[derive(Clone, Deserialize)]
pub struct SpaceConfig {
    pub name: String,
    pub display: String,
    pub path: PathBuf,
}

impl Into<Space> for SpaceConfig {
    fn into(self) -> Space {
        Space {
            name: self.name,
            display: self.display,
            path: self.path,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct UserSpaceConfig {
    name: String,
    permissions: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub email: Option<String>,
    pub spaces: Vec<UserSpaceConfig>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    // global settings
    pub directory_meta_filename: String,

    // application settings
    pub home_path: PathBuf,
    pub users_path: PathBuf,
    pub spaces_path: PathBuf,

    pub spaces: HashMap<String, SpaceConfig>,
    pub users: HashMap<String, UserConfig>,
}

impl Config {
    pub async fn new() -> Result<Self> {
        let mut builder = ConfigBuilder::new();
        let home_path = var("CABY_HOME_PATH").map_err(|err| anyhow!("missing CABY_HOME_PATH"))?;

        // Load minimal settings from env
        builder
            .try_set_dir_meta_filename(var("CABY_DIRECTORY_META_FILENAME").ok())?
            .try_set_home_path(Some(home_path))?
            .try_set_users_path(var("CABY_USERS_PATH").ok())?
            .try_set_spaces_path(var("CABY_SPACES_PATH").ok())?;

        // Load from config
        let config_file = ConfigFile::new_from_path(get_config_path()?).await?;
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

        builder.try_set_users(Some(
            config_file.users.into_iter().map(|u| u.into()).collect(),
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
    spaces: HashMap<String, SpaceConfig>,
    users: HashMap<String, UserConfig>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        //
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
        self.try_set_users_path(Some(pb.join("users")));
        self.try_set_spaces_path(Some(pb.join("spaces")));
        return Ok(self);
    }

    pub fn try_set_users_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        self.users_path = Some(p.into());
        return Ok(self);
    }

    pub fn try_set_spaces_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        self.spaces_path = Some(p.into());
        return Ok(self);
    }

    pub fn try_set_spaces(&mut self, spaces: Option<Vec<SpaceConfig>>) -> Result<&mut Self> {
        let Some(sv) = spaces else {
            return Ok(self);
        };
        sv.iter().for_each(|s| {
            self.spaces.insert(s.name.clone(), s.clone());
        });
        return Ok(self);
    }

    pub fn try_set_users(&mut self, users: Option<Vec<UserConfig>>) -> Result<&mut Self> {
        let Some(uv) = users else {
            return Ok(self);
        };
        uv.iter().for_each(|u| {
            self.users.insert(u.name.clone(), u.clone());
        });
        return Ok(self);
    }

    pub fn build(self) -> Result<Config> {
        return Ok(Config {
            directory_meta_filename: self.directory_meta_filename.ok_or(anyhow!(
                "missing directory meta filename (CABY_DIRECTORY_META_FILENAME)"
            ))?,
            home_path: self.home_path.ok_or(anyhow!("missing home path"))?,
            users_path: self.users_path.ok_or(anyhow!("missing users path"))?,
            spaces_path: self.spaces_path.ok_or(anyhow!("missing spaces path"))?,
            spaces: self.spaces,
            users: self.users,
        });
    }
}
