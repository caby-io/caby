use crate::{Error, Result};
use anyhow::anyhow;
use std::{
    env::var,
    path::{Path, PathBuf},
    sync::Arc,
};
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

    Err(anyhow!("couldn't find a valid config location")
        .context(format!("CABY_HOME_PATH: '{:?}'", var("CABY_HOME_PATH")))
        .context(format!("CABY_CONFIG_PATH: '{:?}'", var("CABY_CONFIG_PATH"))))
}

impl ConfigFile {
    pub async fn new_from_path(path: PathBuf) -> Result<ConfigFile> {
        let mut content = fs::read_to_string(&path).await?;
        let docs = YamlLoader::load_from_str(&content).map_err(|err| {
            return anyhow!("could not load from config file at '{:?}'", path).context(err);
        })?;

        if docs.len() < 1 {
            return Ok(ConfigFile::default());
        }
        let config_yaml = &docs[0];

        for space in config_yaml["spaces"]
            .as_vec()
            .ok_or(anyhow!(".spaces is not an array or is empty"))?
        {
            let name = space["name"]
                .as_str()
                .ok_or(anyhow!("a space is missing a string name"))?;

            println!("{}", name);
        }

        Ok(ConfigFile::default())
    }
}
