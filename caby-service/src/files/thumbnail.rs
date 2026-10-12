use std::{
    fs::Metadata,
    io,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use libvips::ops;
use tokio::{fs, sync::Semaphore, task};

use crate::{
    space::{Space, SpaceDir},
    Result,
};

pub const IMG_THUMB_FILENAME: &str = "thumb.webp";
const MAX_CONCURRENT_THUMBS: usize = 2;
const IMG_MIMES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/heic",
    "image/heif",
    "image/avif",
];

static THUMB_SLOTS: Semaphore = Semaphore::const_new(MAX_CONCURRENT_THUMBS);

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

// todo: explore other ways to track staleness via content
fn is_thumb_stale(source: &Metadata, thumb: io::Result<Metadata>) -> bool {
    let Ok(thumb) = thumb else {
        return true;
    };
    match (source.modified(), thumb.modified()) {
        (Ok(source_modified), Ok(thumb_modified)) => thumb_modified < source_modified,
        _ => false,
    }
}

pub fn needs_thumb(space: &Space, rel: &Path) -> bool {
    let (Ok(live_path), Ok(thumb_path)) = (space.join(SpaceDir::LIVE, rel), thumb_path(space, rel))
    else {
        return false;
    };
    let Ok(source) = std::fs::metadata(&live_path) else {
        return false;
    };

    is_thumb_stale(&source, std::fs::metadata(&thumb_path))
}

pub async fn try_ensure_thumb(space: &Space, rel: &Path, max_edge: u32) -> Result<()> {
    let live_path = space.join(SpaceDir::LIVE, rel)?;
    let thumb_path = thumb_path(space, rel)?;

    let source = match fs::metadata(&live_path).await {
        Ok(source) => source,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(anyhow!(err).context("stat source image")),
    };
    if !is_thumb_stale(&source, fs::metadata(&thumb_path).await) {
        return Ok(());
    }

    match try_generate_thumb(&live_path, &thumb_path, max_edge).await {
        Ok(()) | Err(ThumbError::UnsupportedFormat) => Ok(()),
        Err(ThumbError::Other(err)) => Err(err),
    }
}

pub async fn try_generate_thumb(
    live_path: &Path,
    thumb_path: &Path,
    max_edge: u32,
) -> std::result::Result<(), ThumbError> {
    let live_str = live_path
        .to_str()
        .ok_or_else(|| ThumbError::Other(anyhow!("non-UTF8 source image path")))?
        .to_owned();

    // Unique tmp name so concurrent generators never collide on the same .tmp file.
    let tmp_path = thumb_path.with_extension(format!("webp.tmp.{}", xid::new()));
    let tmp_str = tmp_path
        .to_str()
        .ok_or_else(|| ThumbError::Other(anyhow!("non-UTF8 thumb tmp path")))?
        .to_owned();
    let thumb_dir = thumb_path.parent().map(Path::to_path_buf);

    let permit = THUMB_SLOTS
        .acquire()
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("acquire thumbnail slot")))?;

    let width = max_edge as i32;
    let blocking = task::spawn_blocking(move || -> std::result::Result<(), ThumbError> {
        let _permit = permit;

        let kind = infer::get_from_path(&live_str)
            .map_err(|err| ThumbError::Other(anyhow!(err).context("read source image header")))?
            .ok_or(ThumbError::UnsupportedFormat)?;
        if !IMG_MIMES.contains(&kind.mime_type()) {
            return Err(ThumbError::UnsupportedFormat);
        }

        if let Some(thumb_dir) = thumb_dir {
            std::fs::create_dir_all(thumb_dir)
                .map_err(|err| ThumbError::Other(anyhow!(err).context("mkdir thumb parent")))?;
        }

        // thumbnail decodes from the file (shrinking on load where the format allows), resizes
        // within max width, applies EXIF orientation (no_rotate defaults to false).
        let img = ops::thumbnail(&live_str, width)
            .map_err(|e| ThumbError::Other(anyhow!("libvips thumbnail: {:?}", e)))?;
        ops::webpsave(&img, &tmp_str)
            .map_err(|e| ThumbError::Other(anyhow!("libvips webpsave: {:?}", e)))?;
        Ok(())
    })
    .await
    .map_err(|err| ThumbError::Other(anyhow!(err).context("thumbnail task panicked")))?;

    if let Err(err) = blocking {
        // Clean up the tmp file on failure; ignore errors from the cleanup itself.
        let _ = fs::remove_file(&tmp_path).await;
        return Err(err);
    }

    fs::rename(&tmp_path, thumb_path)
        .await
        .map_err(|err| ThumbError::Other(anyhow!(err).context("publish thumb via rename")))?;

    if !fs::try_exists(live_path).await.unwrap_or(true) {
        let _ = fs::remove_file(thumb_path).await;
        if let Some(thumb_dir) = thumb_path.parent() {
            let _ = fs::remove_dir(thumb_dir).await;
        }
        return Err(ThumbError::Other(anyhow!(
            "source image was removed while its thumbnail was generated"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;

    fn metadata_at(dir: &Path, name: &str, modified: SystemTime) -> Metadata {
        let path = dir.join(name);
        let file = std::fs::File::create(&path).unwrap();
        file.set_modified(modified).unwrap();
        std::fs::metadata(&path).unwrap()
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("caby-thumbs-{}-{}", name, xid::new()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_thumb_is_stale() {
        let dir = scratch_dir("missing");
        let source = metadata_at(&dir, "rocinante.jpg", SystemTime::now());
        let missing = std::fs::metadata(dir.join("thumb.webp"));
        assert!(is_thumb_stale(&source, missing));
    }

    #[test]
    fn thumb_older_than_source_is_stale() {
        let dir = scratch_dir("older");
        let now = SystemTime::now();
        let source = metadata_at(&dir, "rocinante.jpg", now);
        let thumb = metadata_at(&dir, "thumb.webp", now - Duration::from_secs(60));
        assert!(is_thumb_stale(&source, Ok(thumb)));
    }

    #[test]
    fn thumb_newer_than_source_is_fresh() {
        let dir = scratch_dir("newer");
        let now = SystemTime::now();
        let source = metadata_at(&dir, "rocinante.jpg", now - Duration::from_secs(60));
        let thumb = metadata_at(&dir, "thumb.webp", now);
        assert!(!is_thumb_stale(&source, Ok(thumb)));
    }
}
