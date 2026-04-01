use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use tokio::fs;
use yaml_rust2::{yaml, Yaml, YamlEmitter, YamlLoader};

use crate::Result;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserInitState {
    Ready,
    InProgress,
    Completed,
}

#[derive(Clone, Deserialize)]
pub enum InitMethod {
    Code,
    Email,
}

impl std::fmt::Display for InitMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code => write!(f, "code"),
            Self::Email => write!(f, "email"),
        }
    }
}

impl FromStr for InitMethod {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        match input {
            "code" => Ok(InitMethod::Code),
            "email" => Ok(InitMethod::Email),
            _ => Err(anyhow!("input didn't match any known methods: {}", input)),
        }
    }
}

pub struct InitFileCode {
    pub value: String,
    pub attempts: i64,
    pub created_at: DateTime<FixedOffset>,
}

impl TryFrom<&Yaml> for InitFileCode {
    type Error = anyhow::Error;

    fn try_from(value: &Yaml) -> Result<Self> {
        return Ok(Self {
            value: value["value"]
                .as_str()
                .ok_or(anyhow!(".code.value is not a string or is missing"))?
                .to_string(),
            attempts: value["attempts"]
                .as_i64()
                .ok_or(anyhow!(".code.attempts is not a number or is missing"))?,
            created_at: DateTime::parse_from_rfc3339(
                value["created_at"]
                    .as_str()
                    .ok_or(anyhow!(".code.created_at is not a string or is missing"))?,
            )
            .map_err(|err| {
                anyhow!("could not parse .code.created_at as a rfc3339 datetime").context(err)
            })?,
        });
    }
}

pub struct InitFileEmail {
    pub value: String,
    pub attempts: i64,
    pub created_at: DateTime<FixedOffset>,
}

impl TryFrom<&Yaml> for InitFileEmail {
    type Error = anyhow::Error;

    fn try_from(value: &Yaml) -> Result<Self> {
        return Ok(Self {
            value: value["value"]
                .as_str()
                .ok_or(anyhow!(".email.value is not a string or is missing"))?
                .to_string(),
            attempts: value["attempts"]
                .as_i64()
                .ok_or(anyhow!(".email.attempts is not a number or is missing"))?,
            created_at: DateTime::parse_from_rfc3339(
                value["created_at"]
                    .as_str()
                    .ok_or(anyhow!(".email.created_at is not a string or is missing"))?,
            )
            .map_err(|err| {
                anyhow!("could not parse .email.created_at as a rfc3339 datetime").context(err)
            })?,
        });
    }
}

pub struct UserInitFile {
    pub method: InitMethod,
    pub locked_until: Option<DateTime<FixedOffset>>,
    pub code: Option<InitFileCode>,
    pub email: Option<InitFileEmail>,
}

impl UserInitFile {
    pub async fn load(path: &Path) -> Result<Self> {
        let mut content = fs::read_to_string(&path).await.map_err(|err| {
            return anyhow!("could not read init file at {:?}", path).context(err);
        })?;
        let docs = YamlLoader::load_from_str(&content).map_err(|err| {
            return anyhow!("could not parse init file as yaml").context(err);
        })?;

        if docs.len() < 1 {
            return Err(anyhow!("init file is empty"));
        }

        let init_yaml = &docs[0];

        let method = InitMethod::from_str(
            init_yaml["method"]
                .as_str()
                .ok_or(anyhow!("init file is missing a method"))?,
        )
        .map_err(|err| anyhow!("could not convert method to enum").context(err))?;

        let locked_until =
            match init_yaml["locked_until"].as_str() {
                Some(s) => Some(DateTime::parse_from_rfc3339(s).map_err(|err| {
                    anyhow!("could not parse 'locked_until' as date").context(err)
                })?),
                None => None,
            };

        let mut init_file = UserInitFile {
            method: method.clone(),
            locked_until,
            code: None,
            email: None,
        };

        match method {
            InitMethod::Code => init_file.code = Some(InitFileCode::try_from(&init_yaml["code"])?),
            InitMethod::Email => {
                init_file.email = Some(InitFileEmail::try_from(&init_yaml["email"])?)
            }
            _ => return Err(anyhow!("unsupport method")),
        }

        return Ok(init_file);
    }

    pub fn to_yaml(&self) -> String {
        let mut doc = yaml::Hash::new();

        doc.insert(
            Yaml::String("method".into()),
            Yaml::String(self.method.to_string()),
        );

        if let Some(locked) = &self.locked_until {
            doc.insert(
                Yaml::String("locked_until".into()),
                Yaml::String(locked.to_rfc3339()),
            );
        }

        if let Some(code) = &self.code {
            let mut section = yaml::Hash::new();
            section.insert(
                Yaml::String("value".into()),
                Yaml::String(code.value.clone()),
            );
            section.insert(
                Yaml::String("attempts".into()),
                Yaml::Integer(code.attempts),
            );
            section.insert(
                Yaml::String("created_at".into()),
                Yaml::String(code.created_at.to_rfc3339()),
            );
            doc.insert(Yaml::String("code".into()), Yaml::Hash(section));
        }

        if let Some(email) = &self.email {
            let mut section = yaml::Hash::new();
            section.insert(
                Yaml::String("value".into()),
                Yaml::String(email.value.clone()),
            );
            section.insert(
                Yaml::String("attempts".into()),
                Yaml::Integer(email.attempts),
            );
            section.insert(
                Yaml::String("created_at".into()),
                Yaml::String(email.created_at.to_rfc3339()),
            );
            doc.insert(Yaml::String("email".into()), Yaml::Hash(section));
        }

        let mut output = String::new();
        YamlEmitter::new(&mut output)
            .dump(&Yaml::Hash(doc))
            .unwrap();
        return output;
    }

    pub async fn write(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_yaml())
            .await
            .map_err(|err| anyhow!("could not write init file to {:?}", path).context(err))?;
        return Ok(());
    }

    pub fn new_code() -> Self {
        return Self {
            method: InitMethod::Code,
            locked_until: None,
            code: Some(InitFileCode {
                value: format!("{:06}", rand::random_range(0..1_000_000)),
                attempts: 3,
                created_at: chrono::Utc::now().fixed_offset(),
            }),
            email: None,
        };
    }

    pub fn new_email() -> Self {
        return Self {
            method: InitMethod::Email,
            locked_until: None,
            code: None,
            email: Some(InitFileEmail {
                value: xid::new().to_string(),
                attempts: 3,
                created_at: chrono::Utc::now().fixed_offset(),
            }),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn fixed_time() -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc3339("2025-12-15T02:59:43+00:00").unwrap()
    }

    #[test]
    fn to_yaml_code() {
        let init = UserInitFile {
            method: InitMethod::Code,
            locked_until: None,
            code: Some(InitFileCode {
                value: "123456".into(),
                attempts: 3,
                created_at: fixed_time(),
            }),
            email: None,
        };

        let yaml = init.to_yaml();
        let docs = YamlLoader::load_from_str(&yaml).unwrap();
        let doc = &docs[0];

        assert_eq!(doc["method"].as_str().unwrap(), "code");
        assert_eq!(doc["code"]["value"].as_str().unwrap(), "123456");
        assert_eq!(doc["code"]["attempts"].as_i64().unwrap(), 3);
        assert_eq!(
            doc["code"]["created_at"].as_str().unwrap(),
            "2025-12-15T02:59:43+00:00"
        );
        assert!(doc["locked_until"].is_badvalue());
        assert!(doc["email"].is_badvalue());
    }

    #[test]
    fn to_yaml_email() {
        let init = UserInitFile {
            method: InitMethod::Email,
            locked_until: None,
            code: None,
            email: Some(InitFileEmail {
                value: "abc123".into(),
                attempts: 2,
                created_at: fixed_time(),
            }),
        };

        let yaml = init.to_yaml();
        let docs = YamlLoader::load_from_str(&yaml).unwrap();
        let doc = &docs[0];

        assert_eq!(doc["method"].as_str().unwrap(), "email");
        assert_eq!(doc["email"]["value"].as_str().unwrap(), "abc123");
        assert_eq!(doc["email"]["attempts"].as_i64().unwrap(), 2);
        assert!(doc["code"].is_badvalue());
    }

    #[test]
    fn to_yaml_with_locked_until() {
        let init = UserInitFile {
            method: InitMethod::Code,
            locked_until: Some(fixed_time()),
            code: Some(InitFileCode {
                value: "000000".into(),
                attempts: 0,
                created_at: fixed_time(),
            }),
            email: None,
        };

        let yaml = init.to_yaml();
        let docs = YamlLoader::load_from_str(&yaml).unwrap();
        let doc = &docs[0];

        assert_eq!(
            doc["locked_until"].as_str().unwrap(),
            "2025-12-15T02:59:43+00:00"
        );
    }

    #[test]
    fn to_yaml_roundtrips_through_loader() {
        let init = UserInitFile {
            method: InitMethod::Code,
            locked_until: None,
            code: Some(InitFileCode {
                value: "999999".into(),
                attempts: 3,
                created_at: fixed_time(),
            }),
            email: None,
        };

        let yaml = init.to_yaml();
        let docs = YamlLoader::load_from_str(&yaml).unwrap();
        let doc = &docs[0];

        let parsed_code = InitFileCode::try_from(&doc["code"]).unwrap();
        assert_eq!(parsed_code.value, "999999");
        assert_eq!(parsed_code.attempts, 3);
        assert_eq!(parsed_code.created_at, fixed_time());
    }
}
