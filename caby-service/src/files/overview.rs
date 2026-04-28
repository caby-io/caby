use std::path::Path;

use anyhow::anyhow;
use futures_util::TryFutureExt;
use serde::Serialize;
use tokio::fs::{read_dir, DirEntry};
use tracing::error;

use crate::{files::EntryType, space::Space, Result};

#[derive(Serialize)]
pub struct OverviewEntry {
    pub entry_type: EntryType,
    pub name: String,
    pub path: String, // relative path of the file from the mount root
    pub children: Vec<OverviewEntry>,
}

impl OverviewEntry {
    async fn try_from(live_path: &Path, value: DirEntry) -> Result<Self> {
        let metadata = value.metadata().await?;

        // Fill common fields
        // todo: fill in
        let created_at = metadata.created().ok();
        let modified_at = metadata.modified().ok();

        let entry = Self {
            entry_type: match metadata.is_dir() {
                true => EntryType::Directory,
                false => EntryType::File,
            },
            name: value.file_name().into_string().map_err(|err| {
                anyhow!("couldn't convert entry name to string")
                    .context(anyhow!("{:?}", err))
            })?,
            path: value
                .path()
                .strip_prefix(live_path)
                .map_err(|err| anyhow!(err).context("could not strip prefix"))?
                .to_str()
                .ok_or(anyhow!("could not convert root path to string"))?
                .to_owned(),
            // todo: common fields
            children: vec![],
        };

        Ok(entry)
    }
}

pub async fn build_overview(
    space: &Space,
    path: &Path,
    max_depth: u32,
    dirs_only: bool,
) -> Result<Vec<OverviewEntry>> {
    let live_path = space.live();
    let mut entries = read_dir(live_path.join(path)).await?;

    let mut result = vec![];
    // todo: filter earlier to save on compute
    while let Some(dir_entry) = entries.next_entry().await? {
        let filename = dir_entry.file_name();
        let mut entry = match OverviewEntry::try_from(&live_path, dir_entry).await {
            Ok(e) => e,
            Err(err) => {
                error!("couldn't process file: {:?} {:#}", filename, err);
                return Err(err);
            }
        };

        // todo: check that it's a dir
        if matches!(entry.entry_type, EntryType::Directory) && max_depth > 1 {
            let entry_path = live_path.join(entry.path.clone());
            entry.children =
                Box::pin(build_overview(space, &entry_path, max_depth - 1, dirs_only)).await?;
        }

        if dirs_only && !matches!(entry.entry_type, EntryType::Directory) {
            continue;
        }

        result.push(entry);
    }

    result.sort_by(|a, b| {
        let a_key = match a.entry_type {
            EntryType::Directory => format!("0_{}", a.name),
            _ => {
                format!("1_{}", a.name)
            }
        };

        let b_key = match b.entry_type {
            EntryType::Directory => format!("0_{}", b.name),
            _ => {
                format!("1_{}", b.name)
            }
        };

        a_key.cmp(&b_key)
    });

    Ok(result)
}
