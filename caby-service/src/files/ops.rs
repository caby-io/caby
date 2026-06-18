use std::path::Path;

use anyhow::{anyhow, Context};
use tokio::{fs, io};
use tracing::warn;

use crate::{
    error::Result,
    space::{Space, SpaceDir},
};

// Remove/Delete

async fn remove_meta_dir(space: &Space, rel: &Path) {
    let Ok(meta_path) = space.join(SpaceDir::META, rel) else {
        return;
    };
    if let Err(err) = fs::remove_dir_all(&meta_path).await {
        if err.kind() != io::ErrorKind::NotFound {
            warn!("could not remove meta dir for {:?}: {:#}", rel, err);
        }
    }
}

pub async fn remove(space: &Space, rel: &Path) -> Result<()> {
    let live = space.join(SpaceDir::LIVE, rel)?;

    let metadata = fs::metadata(&live).await.context("not found")?;
    if metadata.is_dir() {
        fs::remove_dir_all(&live).await?;
    } else {
        fs::remove_file(&live).await?;
    }

    remove_meta_dir(space, rel).await;
    Ok(())
}

// Rename/Move

async fn rename_meta_dir(space: &Space, src: &Path, dst: &Path) {
    let (Ok(meta_src), Ok(meta_dst)) = (
        space.join(SpaceDir::META, src),
        space.join(SpaceDir::META, dst),
    ) else {
        return;
    };

    if !fs::try_exists(&meta_src).await.unwrap_or(false) {
        return;
    }

    let result = async {
        if let Some(parent) = meta_dst.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::rename(&meta_src, &meta_dst).await
    }
    .await;

    if let Err(err) = result {
        warn!("could not move meta dir {:?} -> {:?}: {:#}", src, dst, err);
    }
}

pub async fn rename(space: &Space, src: &Path, dst: &Path) -> Result<()> {
    let live_src = space.join(SpaceDir::LIVE, src)?;
    let live_dst = space.join(SpaceDir::LIVE, dst)?;

    fs::metadata(&live_src).await.context("source not found")?;

    if fs::try_exists(&live_dst)
        .await
        .context("could not check if destination exists")?
    {
        return Err(anyhow!("destination exists"));
    }

    fs::rename(&live_src, &live_dst).await?;

    rename_meta_dir(space, src, dst).await;
    Ok(())
}
