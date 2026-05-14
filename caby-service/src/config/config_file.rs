use crate::{
    config::{SpaceConfig, UserConfig, UserSpaceConfig},
    validation::{
        self,
        prefabs::{activation_token_validation, email_validation, username_validation},
    },
    Result,
};
use anyhow::{anyhow, Context};
use nest_struct::nest_struct;
use std::{
    collections::HashSet,
    env::var,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::warn;
use yaml_rust2::{Yaml, YamlLoader};

// TODO: saphyr

const CONFIG_FILE_NAME: &str = "config.yaml";

#[derive(Default)]
#[nest_struct]
pub struct ConfigFile {
    pub auth: Option<
        nest! {
            pub struct ConfigFileAuth {
                pub passwords: Option<nest! {
                    enabled: Option<bool>
                }>,
                pub oidc: Option<nest! {
                    pub struct ConfigFileOidc {
                        pub client_id: Option<String>,
                        pub client_secret: Option<String>,
                        pub redirect_uri: Option<String>,
                        pub post_login_redirect: Option<String>,
                        pub scopes: Option<Vec<String>>,
                        pub issuer_url: Option<String>,
                        pub authorization_endpoint: Option<String>,
                        pub token_endpoint: Option<String>,
                        pub jwks_uri: Option<String>,
                        pub userinfo_endpoint: Option<String>,
                    }
                }>,
            }
        },
    >,
    pub spaces: Vec<
        nest! {
            pub struct ConfigFileSpace {
                pub name: String,
                pub display: Option<String>,
                pub path: Option<PathBuf>,
            }
        },
    >,
    pub users: Vec<
        nest! {
            pub struct ConfigFileUser {
                pub name: String,
                pub email: Option<String>,
                pub activation_token: Option<String>,
                pub spaces: Vec<UserSpaceConfig>,
            }
        },
    >,
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

fn parse_auth_section(config_yaml: &Yaml) -> Result<Option<ConfigFileAuth>> {
    let _ = config_yaml;
    return Ok(None);
}

fn parse_spaces_section(config_yaml: &Yaml) -> Result<Vec<ConfigFileSpace>> {
    let mut spaces = vec![];
    let mut spacenames: HashSet<String> = HashSet::new();

    let spaces_yaml = match &config_yaml["spaces"] {
        Yaml::Array(arr) => arr,
        Yaml::BadValue | Yaml::Null => return Err(anyhow!(".spaces is required")),
        _ => return Err(anyhow!(".spaces must be an array")),
    };

    for (i, space) in spaces_yaml.iter().enumerate() {
        let name = match &space["name"] {
            Yaml::String(s) => s.as_str(),
            Yaml::BadValue | Yaml::Null => return Err(anyhow!(".spaces[{}].name is required", i)),
            _ => return Err(anyhow!(".spaces[{}].name must be a string", i)),
        };

        if !spacenames.insert(name.to_string()) {
            return Err(anyhow!("duplicate space name '{}' at .spaces[{}]", name, i));
        }

        let display = match &space["display"] {
            Yaml::BadValue | Yaml::Null => None,
            Yaml::String(s) => Some(s.clone()),
            _ => return Err(anyhow!(".spaces.{}.display must be a string", name)),
        };

        let path = match &space["path"] {
            Yaml::BadValue | Yaml::Null => None,
            Yaml::String(s) => Some(PathBuf::from(s)),
            _ => return Err(anyhow!(".spaces.{}.path must be a string", name)),
        };

        spaces.push(ConfigFileSpace {
            name: name.to_string(),
            display,
            path,
        });
    }

    Ok(spaces)
}

fn parse_users_section(
    config_yaml: &Yaml,
    spaces: &[ConfigFileSpace],
) -> Result<Vec<ConfigFileUser>> {
    let mut users = vec![];
    let mut usernames: HashSet<String> = HashSet::new();

    let users_yaml = match &config_yaml["users"] {
        Yaml::Array(arr) => arr,
        Yaml::BadValue | Yaml::Null => return Err(anyhow!(".users section is missing")),
        _ => return Err(anyhow!(".users must be an array")),
    };

    for (i, user) in users_yaml.iter().enumerate() {
        let name = match &user["name"] {
            Yaml::String(s) => s.as_str(),
            Yaml::BadValue | Yaml::Null => return Err(anyhow!(".users[{}].name is missing", i)),
            _ => return Err(anyhow!(".users[{}].name must be a string", i)),
        };
        if let Some(errs) = validation::exec_stack(&username_validation(), name) {
            return Err(
                anyhow!("{}", errs).context(format!("username '{}' failed validation", name))
            );
        };

        if !usernames.insert(name.to_lowercase()) {
            return Err(anyhow!("duplicate user name '{}' at .users[{}]", name, i));
        }

        let email = match &user["email"] {
            Yaml::BadValue | Yaml::Null => None,
            Yaml::String(s) => Some(s.clone()),
            _ => return Err(anyhow!(".users.{}.email must be a string", name)),
        };
        if let Some(errs) = validation::exec_stack_optional(&email_validation(), email.as_deref()) {
            return Err(anyhow!("{}", errs).context(format!(
                ".users.{}.email: '{}' failed validation",
                name,
                email.as_ref().unwrap()
            )));
        }

        let activation_token = match &user["activation_token"] {
            Yaml::BadValue | Yaml::Null => None,
            Yaml::String(s) => Some(s.clone()),
            _ => return Err(anyhow!(".users.{}.activation_token must be a string", name)),
        };
        if let Some(errs) = validation::exec_stack_optional(
            &activation_token_validation(),
            activation_token.as_deref(),
        ) {
            return Err(anyhow!("{}", errs).context(format!(
                ".users.{}.activation_token: '{}' failed validation",
                name,
                activation_token.as_ref().unwrap()
            )));
        }

        let mut user_spaces = vec![];

        let user_spaces_yaml = match &user["spaces"] {
            Yaml::Array(arr) => arr,
            Yaml::BadValue | Yaml::Null => {
                return Err(anyhow!(".users.{}.spaces section is missing", name))
            }
            _ => return Err(anyhow!(".users.{}.spaces must be an array/list", name)),
        };

        for (j, space) in user_spaces_yaml.iter().enumerate() {
            let space_name = match &space["name"] {
                Yaml::String(s) => s.as_str(),
                Yaml::BadValue | Yaml::Null => {
                    return Err(anyhow!(".users.{}.spaces[{}].name is missing", name, j))
                }
                _ => {
                    return Err(anyhow!(
                        ".users.{}.spaces[{}].name must be a string",
                        name,
                        j
                    ))
                }
            };

            if !spaces.iter().any(|s| s.name == space_name) {
                warn!(
                    "user '{}' has access configuration to a space that does not exist: '{}'",
                    name, space_name
                )
            }

            user_spaces.push(UserSpaceConfig {
                name: space_name.to_string(),
                permissions: vec![],
            });
        }

        users.push(ConfigFileUser {
            name: name.to_string(),
            email,
            activation_token,
            spaces: user_spaces,
        })
    }

    Ok(users)
}

impl ConfigFile {
    pub async fn new_from_path(path: PathBuf) -> Result<ConfigFile> {
        let content = fs::read_to_string(&path).await.map_err(|err| {
            anyhow!(err).context(format!("could not read config file {:?}", path))
        })?;
        let docs = YamlLoader::load_from_str(&content)
            .map_err(|err| anyhow!(err).context("could not parse config file as yaml"))?;

        if docs.is_empty() {
            return Err(anyhow!("config file is empty"));
        }

        let config_yaml = &docs[0];

        let invalid_config_context = || format!("invalid config {:?}", path);
        let auth = parse_auth_section(config_yaml).with_context(invalid_config_context)?;
        let spaces = parse_spaces_section(config_yaml).with_context(invalid_config_context)?;
        let users =
            parse_users_section(config_yaml, &spaces).with_context(invalid_config_context)?;

        Ok(ConfigFile {
            auth,
            spaces,
            users,
        })
    }
}
