use crate::{
    config::{SpaceConfig, UserConfig, UserSpaceConfig},
    validation::{
        exec_stack, exec_stack_optional,
        prefabs::{activation_token_validation, email_validation, username_validation},
    },
    Result,
};
use anyhow::anyhow;
use std::{
    collections::HashMap,
    env::var,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::warn;
use yaml_rust2::YamlLoader;

const CONFIG_FILE_NAME: &str = "config.yaml";

pub struct ConfigFileSpace {
    pub name: String,
    pub display: Option<String>,
    pub path: Option<PathBuf>,
}

impl ConfigFileSpace {
    pub fn into_space_config(self, spaces_path: &Path) -> SpaceConfig {
        SpaceConfig {
            name: self.name.clone(),
            display: self.display.unwrap_or(self.name.clone()).clone(),
            path: self.path.unwrap_or(spaces_path.join(self.name)),
        }
    }
}

pub struct ConfigFileUser {
    pub name: String,
    pub email: Option<String>,
    pub activation_token: Option<String>,
    pub spaces: Vec<UserSpaceConfig>,
}

impl ConfigFileUser {
    pub fn into_user_config(self, users_path: &Path) -> UserConfig {
        UserConfig {
            name: self.name.clone(),
            path: users_path.join(self.name),
            email: self.email,
            activation_token: self.activation_token,
            spaces: self.spaces,
        }
    }
}

#[derive(Default)]
pub struct ConfigFile {
    pub spaces: Vec<ConfigFileSpace>,
    pub users: Vec<ConfigFileUser>,
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
    // todo: break up parsing from I/O for unit testing
    pub async fn new_from_path(path: PathBuf) -> Result<ConfigFile> {
        let content = fs::read_to_string(&path).await.map_err(|err| {
            anyhow!(err).context(format!("could not read config file at {:?}", path))
        })?;
        let docs = YamlLoader::load_from_str(&content)
            .map_err(|err| anyhow!(err).context("could not parse config file as yaml"))?;

        if docs.is_empty() {
            return Err(anyhow!("config file is empty"));
        }

        let mut config_file = ConfigFile::default();
        let config_yaml = &docs[0];

        // parse spaces
        let mut spacenames: HashMap<String, ()> = HashMap::new();
        for space in config_yaml["spaces"]
            .as_vec()
            .ok_or(anyhow!(".spaces is not an array or is missing"))?
        {
            let name = space["name"]
                .as_str()
                .ok_or(anyhow!("a space is missing a string name"))?;

            // Check for space name
            if spacenames.insert(name.to_string(), ()).is_some() {
                return Err(anyhow!(".spaces.name: {}", name).context("duplicate space name"));
            }

            let display = space["display"].as_str().map(|d| d.to_string());

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
                display,
                path,
            });
        }

        // parse users
        let mut usernames: HashMap<String, ()> = HashMap::new();
        for user in config_yaml["users"]
            .as_vec()
            .ok_or(anyhow!(".users is not an array or is missing"))?
        {
            let name = user["name"]
                .as_str()
                .ok_or(anyhow!("a user is missing a string name"))?;
            if let Some(errs) = exec_stack(&username_validation(), name) {
                return Err(
                    anyhow!("{}", errs).context(format!("username '{}' failed validation", name))
                );
            };

            // Check for unique username
            if usernames.insert(name.to_lowercase(), ()).is_some() {
                return Err(anyhow!(".users.name: {}", name).context("duplicate user name"));
            }

            let email = user["email"].as_str();
            if let Some(errs) = exec_stack_optional(&email_validation(), email) {
                return Err(anyhow!("{}", errs).context(format!(
                    ".users.{}.email: '{}' failed validation",
                    name,
                    email.unwrap()
                )));
            }
            let email = email.map(|e| e.to_string());

            let activation_token = user["activation_token"].as_str();
            if let Some(errs) =
                exec_stack_optional(&activation_token_validation(), activation_token)
            {
                return Err(anyhow!("{}", errs).context(format!(
                    ".users.{}.activation_token: '{}' failed validation",
                    name,
                    activation_token.unwrap()
                )));
            }
            let activation_token = activation_token.map(|e| e.to_string());

            // User space access
            let mut spaces = vec![];

            for space in user["spaces"].as_vec().ok_or(anyhow!(
                ".users.{}.spaces is not an array or is missing",
                name
            ))? {
                let space_name = space["name"]
                    .as_str()
                    .ok_or(anyhow!("a user's space access is missing a name"))?;

                let permissions = match space["permissions"].as_vec() {
                    Some(space_permissions) => space_permissions
                        .iter()
                        .map(|s| {
                            s.as_str()
                                .ok_or(anyhow!(
                                    ".users.{}.spaces.{}.permissions has a non-string permission",
                                    name,
                                    space_name
                                ))
                                .map(|s| s.to_string())
                        })
                        .collect(),
                    None => Ok(vec![]),
                }?;

                if !config_file.spaces.iter().any(|s| s.name == space_name) {
                    warn!(
                        "user '{}' has access configuration to a space that does not exist: '{}'",
                        name, space_name
                    )
                }

                spaces.push(UserSpaceConfig {
                    name: space_name.to_string(),
                    permissions,
                });
            }

            config_file.users.push(ConfigFileUser {
                name: name.to_string(),
                email,
                activation_token,
                spaces,
            })
        }

        Ok(config_file)
    }
}
