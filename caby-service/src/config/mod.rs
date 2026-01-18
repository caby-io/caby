use crate::{Error, Result};
use serde::Deserialize;
use std::{
    env::var,
    path::{Path, PathBuf},
};

pub mod config_env;
pub mod config_file;

#[derive(Clone, Deserialize)]
pub struct ConfigPaths {
    pub config_path: PathBuf,
    pub users_path: PathBuf,
    pub spaces_path: PathBuf,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub paths: ConfigPaths,

    // temp
    pub live_path: PathBuf,
    pub uploads_path: PathBuf,
}

impl Config {
    pub fn get_config_path() -> Result<PathBuf> {
        if let Ok(config_path) = var("CABY_CONFIG_PATH") {
            return Ok(PathBuf::from(config_path));
        }

        if let Ok(home_path) = var("CABY_HOME_PATH") {
            return Ok(Path::new(&home_path).join("config/config.yaml"));
        }

        Err(Error::Generic(
            "couldn't find a valid config location".to_string(),
        ))
    }

    pub fn new() -> Self {
        let cfg_path = Config::get_config_path().unwrap();

        let mut builder = ConfigBuilder::new();

        // set defaults
        // todo: no defaults atm

        // set paths in default locations relative to CABY_HOME_PATH
        builder
            .set_config_path(
                var("CABY_HOME_PATH")
                    .map(|s| Path::new(&s).join("config"))
                    .ok(),
            )
            .set_config_path(var("CABY_CONFIG_PATH").ok());

        // todo: try load config file
        // todo: create it if it doesn't exist

        builder.set_meta_path(
            var("CABY_HOME_PATH")
                .map(|s| Path::new(&s).join("meta"))
                .ok(),
        );
        builder.set_uploads_path(
            var("CABY_HOME_PATH")
                .map(|s| Path::new(&s).join("uploads"))
                .ok(),
        );

        // override
        builder.set_live_path(var("CABY_LIVE_PATH").ok());
        builder.set_meta_path(var("CABY_META_PATH").ok());
        builder.set_uploads_path(var("CABY_META_PATH").ok());

        // validate & build
        builder.build()
    }
}

#[derive(Default)]
pub struct ConfigBuilderConfigs {
    pub config_path: Option<PathBuf>,
    pub users_path: Option<PathBuf>,
    pub spaces_path: Option<PathBuf>,
}

#[derive(Default)]
pub struct ConfigBuilder {
    pub paths: ConfigBuilderConfigs,
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

    pub fn set_config_path(&mut self, path: Option<impl Into<PathBuf>>) -> &mut Self {
        ConfigBuilder::try_override(&mut self.paths.config_path, path.map(|p| p.into()));
        return self;
    }

    pub fn set_live_path(&mut self, path: Option<impl Into<PathBuf>>) {
        // ConfigBuilder::override_field(&mut self.live_path, path.map(|p| p.into()))
    }

    pub fn set_meta_path(&mut self, path: Option<impl Into<PathBuf>>) {
        // ConfigBuilder::override_field(&mut self.meta_path, path.map(|p| p.into()))
    }

    pub fn set_uploads_path(&mut self, path: Option<impl Into<PathBuf>>) {
        // ConfigBuilder::override_field(&mut self.uploads_path, path.map(|p| p.into()))
    }

    pub fn build(self) -> Config {
        Config {
            paths: ConfigPaths {
                config_path: PathBuf::from("todo!()"),
                users_path: PathBuf::from("todo!()"),
                spaces_path: PathBuf::from("todo!()"),
            },
            live_path: PathBuf::from("todo!()"),
            uploads_path: PathBuf::from("todo!()"),
        }
    }
}
