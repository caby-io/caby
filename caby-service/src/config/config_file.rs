use crate::{config::SpaceConfig, Error, Result};
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

const CONFIG_FILE_NAME: &'static str = "config.yaml";

pub struct ConfigFileSpace {
    pub name: String,
    pub path: Option<PathBuf>,
}

impl ConfigFileSpace {
    pub fn into_space_config(self, spaces_path: &Path) -> SpaceConfig {
        return SpaceConfig {
            name: self.name.clone(),
            path: self.path.unwrap_or(spaces_path.join(self.name)),
        };
    }
}

#[derive(Default)]
pub struct ConfigFile {
    pub spaces: Vec<ConfigFileSpace>,
}

pub fn get_config_path() -> Result<PathBuf> {
    if let Ok(configs_path) = var("CABY_CONFIG_PATH") {
        return Ok(PathBuf::from(configs_path));
    }

    if let Ok(home_path) = var("CABY_HOME_PATH") {
        return Ok(Path::new(&home_path).join(CONFIG_FILE_NAME));
    }

    Err(anyhow!("could not find a valid config location")
        .context(format!("CABY_HOME_PATH: {:?}", var("CABY_HOME_PATH")))
        .context(format!("CABY_CONFIG_PATH: {:?}", var("CABY_CONFIG_PATH"))))
}

impl ConfigFile {
    pub async fn new_from_path(path: PathBuf) -> Result<ConfigFile> {
        let mut content = fs::read_to_string(&path).await.map_err(|err| {
            return anyhow!("could not read config file at {:?}", path).context(err);
        })?;
        let docs = YamlLoader::load_from_str(&content).map_err(|err| {
            return anyhow!("could not parse config file as yaml").context(err);
        })?;

        if docs.len() < 1 {
            return Err(anyhow!("config file is empty"));
        }

        let mut config_file = ConfigFile::default();

        let config_yaml = &docs[0];

        for space in config_yaml["spaces"]
            .as_vec()
            .ok_or(anyhow!(".spaces is not an array or is empty"))?
        {
            let name = space["name"]
                .as_str()
                .ok_or(anyhow!("a space is missing a string name"))?;

            let path = match space["path"].is_null() {
                true => Some(PathBuf::from(
                    space["path"]
                        .as_str()
                        .ok_or(anyhow!("could not get .spaces.{} path as string", &name))?,
                )),
                false => None,
            };

            config_file.spaces.push(ConfigFileSpace {
                name: name.to_string(),
                path,
            });
        }

        Ok(config_file)
    }
}
