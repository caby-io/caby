use std::{io, path::Path};

use anyhow::anyhow;
use tokio::fs;

use crate::error::Result;

// Move every entry in `src` into `dest`, recursing into directories that already exist
// at the destination. Files at conflicting paths are overwritten (rename semantics on Linux);
// type mismatches (file vs directory) bubble up as errors. `src` is removed when empty.
pub async fn merge_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).await?;
    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_src = entry.path();
        let entry_dst = dst.join(entry.file_name());
        let src_meta = entry.metadata().await?;

        if !src_meta.is_dir() {
            fs::rename(&entry_src, &entry_dst).await?;
            continue;
        }

        match fs::metadata(&entry_dst).await {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                fs::rename(&entry_src, &entry_dst).await?;
            }
            Err(err) => {
                return Err(
                    anyhow!(err).context(format!("could not get metadata for '{:?}'", entry_dst))
                )
            }
            Ok(meta) if meta.is_dir() => {
                Box::pin(merge_dir(&entry_src, &entry_dst)).await?;
            }
            Ok(_) => return Err(anyhow!("src/dst dir mismatch: {:?}", entry_dst)),
        }
        continue;
    }

    // slow cleanup, ensures that directory was "drained"
    fs::remove_dir(src).await?;
    Ok(())
}
