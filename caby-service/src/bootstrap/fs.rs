use std::path::Path;

use anyhow::{anyhow, Ok};
use tokio::fs::{create_dir_all, try_exists};
use tracing::{info, warn};

use crate::{
    auth::oidc::{oidc_user::load_provisioned_users, OIDC_DIR},
    config::Config,
    error::Result,
};

async fn init_dir(name: &str, path: &Path) -> Result<()> {
    let exists = try_exists(path)
        .await
        .map_err(|e| anyhow!("could not validate {} path exists at {:?}", name, path).context(e))?;
    if exists {
        return Ok(());
    }

    create_dir_all(path)
        .await
        .map_err(|e| anyhow!("could not create {} directory at {:?}", name, path).context(e))?;
    info!("created {} path at {:?}", name, path);
    Ok(())
}

pub async fn init(cfg: &Config) -> Result<()> {
    // Initialize core application paths
    init_dir("users", &cfg.users_path).await?;
    init_dir("spaces", &cfg.spaces_path).await?;
    init_dir("shares", &cfg.shares_path).await?;
    if cfg.auth.oidc.is_some() {
        init_dir(OIDC_DIR, &cfg.home_path.join(OIDC_DIR)).await?;
    }

    let users = load_provisioned_users(&cfg.users_path).await?;
    if !users.is_empty() {
        info!("found {} provisioned OIDC users", users.len());
        for (key, user) in users {
            if let Err(err) = cfg.try_upsert_app_user(key, user) {
                warn!("skipping provisioned OIDC user: {}", err);
            }
        }
    }

    // Initialize spaces
    let cfg_rtm = cfg.runtime.load();
    for (_, space_config) in cfg_rtm.spaces.iter() {
        if !space_config.managed {
            continue;
        }

        init_dir(
            &format!("spaces/{}/live", &space_config.name),
            &space_config.live,
        )
        .await?;
        init_dir(
            &format!("spaces/{}/meta", &space_config.name),
            &space_config.meta,
        )
        .await?;
        init_dir(
            &format!("spaces/{}/uploads", &space_config.name),
            &space_config.uploads,
        )
        .await?;
    }

    Ok(())
}
