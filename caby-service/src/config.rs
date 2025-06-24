use std::{
    env::var,
    path::{Path, PathBuf},
};

use serde::Deserialize;

// todo: improve the error messages

#[derive(Clone, Deserialize)]
pub struct Config {
    // paths
    pub live_path: PathBuf,
    pub meta_path: PathBuf,
    pub uploads_path: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        let mut builder = ConfigBuilder::new();

        // set defaults
        // todo: no defaults atm

        // set paths in default locations relative to CABY_HOME_PATH
        builder.set_live_path(
            var("CABY_HOME_PATH")
                .map(|s| Path::new(&s).join("live"))
                .ok(),
        );
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
pub struct ConfigBuilder {
    // paths
    pub live_path: Option<PathBuf>,
    pub meta_path: Option<PathBuf>,
    pub uploads_path: Option<PathBuf>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        //
        Self::default()
    }

    fn override_field<T>(field: &mut Option<T>, value: Option<T>) {
        if value.is_none() {
            return;
        }

        *field = value
    }

    pub fn set_live_path(&mut self, path: Option<impl Into<PathBuf>>) {
        ConfigBuilder::override_field(&mut self.live_path, path.map(|p| p.into()))
    }

    pub fn set_meta_path(&mut self, path: Option<impl Into<PathBuf>>) {
        ConfigBuilder::override_field(&mut self.meta_path, path.map(|p| p.into()))
    }

    pub fn set_uploads_path(&mut self, path: Option<impl Into<PathBuf>>) {
        ConfigBuilder::override_field(&mut self.uploads_path, path.map(|p| p.into()))
    }

    pub fn build(self) -> Config {
        Config {
            live_path: self.live_path.unwrap(),
            meta_path: self.meta_path.unwrap(),
            uploads_path: self.uploads_path.unwrap(),
        }
    }
}
