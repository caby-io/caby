use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use serde::Deserialize;
use tokio::{fs, io};
use tracing::warn;

use crate::{
    error::{Error, Result},
    space::{Space, SpaceDir},
};

pub fn is_name_too_long(err: &Error) -> bool {
    err.downcast_ref::<io::Error>().is_some_and(is_too_long)
}

fn is_too_long(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::InvalidFilename
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum FileConflictStrategy {
    Override,
    Skip,
    Deconflict,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum DirConflictStrategy {
    #[default]
    Merge,
    Skip,
    Deconflict,
}

pub enum WriteOutcome {
    Created(PathBuf),
    Overwritten(PathBuf),
    Deconflicted(PathBuf),
    Skipped(PathBuf),
}

pub async fn write_file(
    space: &Space,
    rel: &Path,
    content: &str,
    strategy: FileConflictStrategy,
) -> Result<WriteOutcome> {
    let live = space.join(SpaceDir::LIVE, rel)?;

    if !fs::try_exists(&live).await? {
        fs::write(&live, content).await?;
        return Ok(WriteOutcome::Created(rel.to_path_buf()));
    }

    match strategy {
        FileConflictStrategy::Override => {
            fs::write(&live, content).await?;
            Ok(WriteOutcome::Overwritten(rel.to_path_buf()))
        }
        FileConflictStrategy::Skip => Ok(WriteOutcome::Skipped(rel.to_path_buf())),
        FileConflictStrategy::Deconflict => {
            let deconflicted = deconflict(space, rel).await?;
            let live = space.join(SpaceDir::LIVE, &deconflicted)?;
            fs::write(&live, content).await?;
            Ok(WriteOutcome::Deconflicted(deconflicted))
        }
    }
}

pub async fn create_dir(
    space: &Space,
    rel: &Path,
    strategy: DirConflictStrategy,
) -> Result<PathBuf> {
    let live = space.join(SpaceDir::LIVE, rel)?;

    if !fs::try_exists(&live).await? {
        fs::create_dir(&live).await?;
        return Ok(rel.to_path_buf());
    }

    match strategy {
        // a freshly-created dir is empty, so merge and skip are both idempotent no-ops
        DirConflictStrategy::Merge | DirConflictStrategy::Skip => Ok(rel.to_path_buf()),
        DirConflictStrategy::Deconflict => {
            let deconflicted = deconflict(space, rel).await?;
            fs::create_dir(space.join(SpaceDir::LIVE, &deconflicted)?).await?;
            Ok(deconflicted)
        }
    }
}

async fn deconflict(space: &Space, rel: &Path) -> Result<PathBuf> {
    let parent = rel.parent().unwrap_or(Path::new(""));
    let stem = rel
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("invalid file name {:?}", rel))?;
    let ext = rel.extension().and_then(|ext| ext.to_str());

    let mut stem = stem.to_string();
    for n in 2..1000 {
        loop {
            let name = match ext {
                Some(ext) => format!("{stem} ({n}).{ext}"),
                None => format!("{stem} ({n})"),
            };
            let candidate = parent.join(name);
            match fs::try_exists(space.join(SpaceDir::LIVE, &candidate)?).await {
                Ok(true) => break,
                Ok(false) => return Ok(candidate),
                Err(err) if is_too_long(&err) => {
                    if stem.pop().is_none() {
                        return Err(anyhow!("could not shorten {:?} to a valid name", rel));
                    }
                }
                Err(err) => return Err(err.into()),
            }
        }
    }

    Err(anyhow!("could not find an available name for {:?}", rel))
}

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
