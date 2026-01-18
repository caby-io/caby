use std::{
    env::var,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{Error, Result};
use tokio::{
    fs::{self, File},
    io::AsyncReadExt,
};
use yaml_rust2::{Yaml, YamlLoader};

pub struct ConfigFileSpace {
    pub name: String,
}

#[derive(Default)]
pub struct ConfigFile {
    pub spaces: Option<Vec<ConfigFileSpace>>,
}

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

impl ConfigFile {
    pub async fn new_from_path(path: PathBuf) -> Result<ConfigFile> {
        let mut content = fs::read_to_string(&path).await?;
        let docs = YamlLoader::load_from_str(&content).map_err(|err| {
            Error::Generic(format!(
                "could not open config file at '{:?}': {}",
                path, err
            ))
        })?;

        if docs.len() < 1 {
            return Ok(ConfigFile::default());
        }
        let config = &docs[0];

        if matches!(config["spaces"], Yaml::Array(_)) {
            config["spaces"]
                .as_vec()
                .unwrap()
                .into_iter()
                .for_each(|space| {
                    println!("{:?}", space["name"].as_str().unwrap());
                    // println!("{:?}", space["test"].as_str().unwrap());
                });
            // config["spaces"].as_vec()
        }

        Ok(ConfigFile::default())
    }
}
