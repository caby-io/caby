use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::{anyhow, Context};
use chrono::Utc;
use openidconnect::core::CoreIdTokenClaims;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    auth::oidc::oidc_user_names::{calculate_name, ADJECTIVES, ANIMALS},
    config::{Config, Runtime, UserConfig},
    user::User,
    Result,
};

pub const OIDC_USER_FILE: &str = "oidc.json";

#[derive(Clone, Serialize, Deserialize)]
pub struct OidcUserFile {
    pub issuer: String,
    pub subject: String,
    pub email: Option<String>,
    pub created_at: String,
}

pub fn resolve_display_name(claims: &CoreIdTokenClaims) -> String {
    claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.as_str().trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| calculate_name(claims.subject().as_str()))
}

fn write_to_config(cfg: &Config, key: String, user_config: UserConfig) {
    cfg.runtime.rcu(|r| {
        let mut next = (**r).clone();
        next.users.insert(key.clone(), user_config.clone());
        Arc::new(next)
    });
}

// this will load users based on the filesystem
// important for users that come from oidc
pub async fn load_provisioned_users(users_path: &Path) -> Result<HashMap<String, UserConfig>> {
    let mut out = HashMap::new();

    let mut dir = match fs::read_dir(users_path).await {
        Ok(d) => d,
        Err(err) => {
            return Err(
                anyhow!(err).context(format!("could not read users path at {:?}", users_path))
            );
        }
    };

    while let Ok(Some(entry)) = dir.next_entry().await {
        let metadata = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !metadata.is_dir() {
            continue;
        }

        let oidc_file = entry.path().join(OIDC_USER_FILE);
        let content = match fs::read_to_string(&oidc_file).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed: OidcUserFile = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(err) => {
                return Err(
                    anyhow!(err).context(format!("could not parse OIDC user file {:?}", oidc_file))
                );
            }
        };

        let user_config = UserConfig {
            name: parsed.subject.clone(),
            path: entry.path(),
            email: parsed.email,
            activation_token: None,
            spaces: vec![],
        };

        out.insert(parsed.subject, user_config);
    }

    Ok(out)
}

pub async fn provision_user(cfg: &Config, claims: &CoreIdTokenClaims) -> Result<User> {
    let subject = claims.subject().to_string();
    let issuer = claims.issuer().to_string();
    let email = claims.email().map(|e| e.to_string());
    let user_path = cfg.users_path.join(&subject);
    let oidc_file_path = user_path.join(OIDC_USER_FILE);

    if fs::try_exists(&oidc_file_path)
        .await
        .with_context(|| format!("could not check OIDC user file {:?}", oidc_file_path))?
    {
        let content = fs::read_to_string(&oidc_file_path)
            .await
            .with_context(|| format!("could not read OIDC user file {:?}", oidc_file_path))?;

        let existing: OidcUserFile = serde_json::from_str(&content)
            .with_context(|| format!("could not parse OIDC user file {:?}", oidc_file_path))?;

        let user_config = UserConfig {
            name: subject.clone(),
            path: user_path,
            email: existing.email,
            activation_token: None,
            spaces: vec![],
        };
        write_to_config(cfg, subject, user_config.clone());
        return Ok((&user_config).into());
    }

    if let Some(ref new_email) = email {
        let rtm = cfg.runtime.load();
        let collision = rtm.users.values().any(|u| {
            u.email
                .as_ref()
                .map(|existing| existing == new_email)
                .unwrap_or(false)
        });
        if collision {
            return Err(anyhow!(
                "OIDC email {} collides with an existing user",
                new_email
            ));
        }
    }

    fs::create_dir_all(&user_path)
        .await
        .with_context(|| format!("could not create user directory {:?}", user_path))?;

    let oidc_file = OidcUserFile {
        issuer,
        subject: subject.clone(),
        email: email.clone(),
        created_at: Utc::now().to_rfc3339(),
    };

    let serialized =
        serde_json::to_string_pretty(&oidc_file).context("could not serialize OIDC user file")?;

    fs::write(&oidc_file_path, serialized)
        .await
        .with_context(|| format!("could not write OIDC user file {:?}", oidc_file_path))?;

    let user_config = UserConfig {
        name: subject.clone(),
        path: user_path,
        email,
        activation_token: None,
        spaces: vec![],
    };
    write_to_config(cfg, subject, user_config.clone());

    Ok((&user_config).into())
}
