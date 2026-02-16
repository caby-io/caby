use std::{
    io::{self, ErrorKind},
    path::Path,
};

use anyhow::anyhow;
use futures_util::TryFutureExt;
use serde::{de, Serialize};
use tokio::fs::{read_dir, DirEntry};
use tracing::error;

use crate::{
    files::{pretty, EntryType},
    space::Space,
    Error, Result,
};

#[derive(Serialize)]
pub struct EntryOverview {
    pub entry_type: EntryType,
    pub name: String,
    pub path: String, // relative path of the file from the mount root
    pub children: Vec<EntryOverview>,
}

impl EntryOverview {
    async fn try_from(root_path: &Path, value: DirEntry) -> Result<Self> {
        let metadata = value.metadata().await?;

        // Fill common fields
        let created_at = metadata.created().ok();
        let modified_at = metadata.modified().ok();

        let mut entry = Self {
            name: value.file_name().into_string().map_err(|err| {
                return anyhow!("couldn't convert entry name to string")
                    .context(anyhow!("{:?}", err));
            })?,
            path: value
                .path()
                .strip_prefix(root_path)
                .map_err(|err| io::Error::new(ErrorKind::Other, "couldn't strip prefix"))?
                .to_str()
                .ok_or(io::Error::new(
                    ErrorKind::Other,
                    "couldn't convert root path to string",
                ))?
                .to_owned(),
            entry_type: match metadata.is_dir() {
                true => EntryType::Directory,
                false => EntryType::File,
            },
            children: vec![],
        };

        return Ok(entry);
    }
}

pub async fn build_overview(
    space: &Space,
    path: &Path,
    depth: u32,
    dirs_only: bool,
) -> Result<Vec<EntryOverview>> {
    let mut entries = read_dir(path).await?;

    let mut result = vec![];

    // todo: filter earlier to save on compute
    while let Some(dir_entry) = entries.next_entry().await? {
        let live_path = space.live();
        let filename = dir_entry.file_name();
        let mut entry = match EntryOverview::try_from(&live_path, dir_entry).await {
            Ok(e) => e,
            Err(err) => {
                error!("couldn't process file: {:?} {}", filename, err);
                return Err(err);
            }
        };

        // todo: check that it's a dir
        if matches!(entry.entry_type, EntryType::Directory) && depth > 1 {
            let entry_path = live_path.join(entry.path.clone());
            entry.children =
                Box::pin(build_overview(space, &entry_path, depth - 1, dirs_only)).await?;
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

    return Ok(result);
}
