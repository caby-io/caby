use crate::Result;
use anyhow::{anyhow, Context, Error};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::var,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

pub mod config_file;

#[derive(Clone, Deserialize)]
pub struct SpaceConfig {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub home_path: PathBuf,
    pub config_path: PathBuf,
    pub users_path: PathBuf,
    pub spaces_path: PathBuf,

    pub spaces: HashMap<String, SpaceConfig>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let mut builder = ConfigBuilder::new();
        let home_path = var("CABY_HOME_PATH").map_err(|err| anyhow!("missing CABY_HOME_PATH"))?;

        // Load from env
        builder
            .try_set_home_path(Some(home_path))?
            .try_set_configs_path(var("CABY_CONFIGS_PATH").ok())?
            .try_set_users_path(var("CABY_USERS_PATH").ok())?
            .try_set_spaces_path(var("CABY_SPACES_PATH").ok())?;

        // Load from config
        // let config_path = Self::get_config_path()?;
        // todo: parse
        // todo: map values

        builder.build()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    home_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    users_path: Option<PathBuf>,
    spaces_path: Option<PathBuf>,
    spaces: HashMap<String, SpaceConfig>,
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

    pub fn try_set_home_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        let pb = p.into();
        self.home_path = Some(pb.clone());
        self.try_set_configs_path(Some(pb.join("/configs")));
        self.try_set_users_path(Some(pb.join("/users")));
        self.try_set_spaces_path(Some(pb.join("/spaces")));
        // todo: set other paths from this
        return Ok(self);
    }

    pub fn try_set_configs_path(&mut self, path: Option<impl Into<PathBuf>>) -> Result<&mut Self> {
        let Some(p) = path else {
            return Ok(self);
        };
        self.config_path = Some(p.into());
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

    pub fn build(self) -> Result<Config> {
        return Ok(Config {
            home_path: self.home_path.ok_or(anyhow!("missing home path"))?,
            config_path: self.config_path.ok_or(anyhow!("missing config path"))?,
            users_path: self.users_path.ok_or(anyhow!("missing users path"))?,
            spaces_path: self.spaces_path.ok_or(anyhow!("missing spaces path"))?,
            spaces: self.spaces,
        });
    }
}
