use std::path::Path;

use anyhow::{anyhow, Ok};
use tokio::fs::{create_dir_all, try_exists};
use tracing::info;

use crate::{config::Config, error::Result};

async fn init_dir(name: &str, path: &Path) -> Result<()> {
    let exists = try_exists(path)
        .await
        .map_err(|e| anyhow!("could not validate {} path exists at {:?}", name, path).context(e))?;
    if (exists) {
        return Ok(());
    }

    create_dir_all(path)
        .await
        .map_err(|e| anyhow!("could not create {} directory at {:?}", name, path).context(e))?;
    info!("created {} path at {:?}", name, path);
    Ok(())
}

pub async fn init(cfg: &Config) -> Result<()> {
    // Initial core application paths
    init_dir("users", &cfg.users_path).await?;
    init_dir("spaces", &cfg.spaces_path).await?;

    // Initialize spaces
    for (_, space_config) in cfg.spaces.iter() {
        init_dir(
            &format!("spaces/{}", &space_config.name),
            &space_config.path,
        )
        .await?;

        init_dir(
            &format!("spaces/{}/live", &space_config.name),
            &space_config.path.join("live"),
        )
        .await?;
        init_dir(
            &format!("spaces/{}/meta", &space_config.name),
            &space_config.path.join("meta"),
        )
        .await?;
        init_dir(
            &format!("spaces/{}/shares", &space_config.name),
            &space_config.path.join("shares"),
        )
        .await?;
        init_dir(
            &format!("spaces/{}/uploads", &space_config.name),
            &space_config.path.join("uploads"),
        )
        .await?;
    }

    Ok(())
}
