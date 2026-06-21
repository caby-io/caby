use std::path::{Path, PathBuf};

use anyhow::anyhow;
use libvips::ops;
use tokio::{fs, task};

use crate::{
    space::{Space, SpaceDir},
    Result,
};

pub const IMG_THUMB_FILENAME: &str = "thumb.webp";

const IMG_MIMES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/heic",
    "image/heif",
    "image/avif",
];

#[derive(Debug)]
pub enum ThumbError {
    UnsupportedFormat,
    Other(anyhow::Error),
}

impl std::fmt::Display for ThumbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbError::UnsupportedFormat => write!(f, "unsupported format"),
            ThumbError::Other(err) => write!(f, "{:#}", err),
        }
    }
}

impl std::error::Error for ThumbError {}

pub fn thumb_path(space: &Space, rel: &Path) -> Result<PathBuf> {
    Ok(space.join(SpaceDir::META, rel)?.join(IMG_THUMB_FILENAME))
}

// todo: Put into task/queue system
pub async fn try_generate_thumb(
    live_path: &Path,
    thumb_path: &Path,
    max_edge: u32,
) -> std::result::Result<(), ThumbError> {
    let bytes = fs::read(live_path)
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("read source image")))?;

    let kind = infer::get(&bytes).ok_or(ThumbError::UnsupportedFormat)?;
    if !IMG_MIMES.contains(&kind.mime_type()) {
        return Err(ThumbError::UnsupportedFormat);
    }

    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|err| ThumbError::Other(anyhow!(err).context("mkdir thumb parent")))?;
    }

    // Unique tmp name so concurrent generators never collide on the same .tmp file.
    let tmp_path = thumb_path.with_extension(format!("webp.tmp.{}", xid::new()));
    let tmp_str = tmp_path
        .to_str()
        .ok_or_else(|| ThumbError::Other(anyhow!("non-UTF8 thumb tmp path")))?
        .to_owned();

    let width = max_edge as i32;
    let blocking = task::spawn_blocking(move || -> std::result::Result<(), String> {
        // thumbnail_buffer decodes, resizes within max width, applies EXIF orientation
        // (no_rotate defaults to false), all in one streaming op.
        let img = ops::thumbnail_buffer(&bytes, width)
            .map_err(|e| format!("libvips thumbnail: {:?}", e))?;
        ops::webpsave(&img, &tmp_str).map_err(|e| format!("libvips webpsave: {:?}", e))?;
        Ok(())
    })
    .await
    .map_err(|err| ThumbError::Other(anyhow!(err).context("thumbnail task panicked")))?;

    if let Err(msg) = blocking {
        // Clean up the tmp file on failure; ignore errors from the cleanup itself.
        let _ = fs::remove_file(&tmp_path).await;
        return Err(ThumbError::Other(anyhow!(msg)));
    }

    fs::rename(&tmp_path, thumb_path)
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("publish thumb via rename")))?;

    Ok(())
}
