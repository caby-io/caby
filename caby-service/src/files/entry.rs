use std::{fs::Metadata, os::unix::fs::MetadataExt, path::Path, time::SystemTime};

use anyhow::anyhow;
use nest_struct::nest_struct;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, read_dir, read_link, DirEntry};
use tracing::{error, warn};

use crate::{config::Config, error::Result, img_thumbs, space::Space};

use super::{
    media_type::{FileKind, MediaType},
    pretty,
};

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    #[default]
    Unknown,
    Directory,
    File,
    Symlink,
}

#[derive(Serialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum EntryFields {
    File {
        size: u64,
        pretty_size: String,
        media_type: Option<MediaType>,
        kind: FileKind,
        preview_url: Option<String>,
    },
    Symlink {
        is_broken: bool,
        target_type: EntryType,
        target_path: String,
    },
}

#[derive(Serialize)]
pub struct Entry {
    pub entry_type: EntryType,

    // common fields
    pub name: String,
    pub path: String, // relative path of the file within the space
    pub created_at: Option<SystemTime>,
    pub pretty_created_at: String,
    pub modified_at: Option<SystemTime>,
    pub pretty_modified_at: String,

    // extra fields
    pub entry_fields: Option<EntryFields>,
}

#[derive(Default)]
#[nest_struct]
struct EntryFactory {
    common: Option<
        nest! {
            struct EntryCommon {
                name: String,
                path: String,
                created_at: Option<SystemTime>,
                pretty_created_at: String,
                modified_at: Option<SystemTime>,
                pretty_modified_at: String,
            }
        },
    >,
    entry_type: Option<EntryType>,
    fields: Option<EntryFields>,
}

impl EntryFactory {
    fn new() -> Self {
        Self::default()
    }

    fn set_common(&mut self, name: String, path: String, metadata: &Metadata) -> &mut Self {
        let created_at = metadata.created().ok();
        let modified_at = metadata.modified().ok();
        self.common = Some(EntryCommon {
            name,
            path,
            pretty_created_at: pretty::date(&created_at),
            pretty_modified_at: pretty::date(&modified_at),
            created_at,
            modified_at,
        });
        self
    }

    fn set_directory(&mut self) -> &mut Self {
        self.entry_type = Some(EntryType::Directory);
        self.fields = None;
        self
    }

    fn set_file(
        &mut self,
        metadata: &Metadata,
        media_type: Option<MediaType>,
        kind: FileKind,
        preview_url: Option<String>,
    ) -> &mut Self {
        let size = metadata.size();
        self.entry_type = Some(EntryType::File);
        self.fields = Some(EntryFields::File {
            size,
            pretty_size: pretty::bytes(size),
            media_type,
            kind,
            preview_url,
        });
        self
    }

    fn set_symlink(
        &mut self,
        is_broken: bool,
        target_type: EntryType,
        target_path: String,
    ) -> &mut Self {
        self.entry_type = Some(EntryType::Symlink);
        self.fields = Some(EntryFields::Symlink {
            is_broken,
            target_type,
            target_path,
        });
        self
    }

    fn build(self) -> Result<Entry> {
        let common = self
            .common
            .ok_or_else(|| anyhow!("entry common fields not set"))?;
        let entry_type = self
            .entry_type
            .ok_or_else(|| anyhow!("entry type not set"))?;

        Ok(Entry {
            entry_type,
            name: common.name,
            path: common.path,
            created_at: common.created_at,
            pretty_created_at: common.pretty_created_at,
            modified_at: common.modified_at,
            pretty_modified_at: common.pretty_modified_at,
            entry_fields: self.fields,
        })
    }
}

async fn build_entry(
    dir_entry: DirEntry,
    live_path: &Path,
    thumb_urls: Option<&img_thumbs::ThumbUrlBuilder<'_>>,
) -> Result<Entry> {
    let metadata = dir_entry.metadata().await?;

    let name = dir_entry.file_name().into_string().map_err(|err| {
        anyhow!("could not convert entry name to string").context(format!("{:?}", err))
    })?;
    let path = dir_entry
        .path()
        .strip_prefix(live_path)
        .map_err(|err| anyhow!(err).context("could not strip prefix"))?
        .to_str()
        .ok_or(anyhow!("could not convert entry path to string"))?
        .to_owned();

    let mut factory = EntryFactory::new();
    factory.set_common(name, path.clone(), &metadata);

    if metadata.is_dir() {
        factory.set_directory();
    } else if metadata.is_file() {
        let media_type = MediaType::from_path(&dir_entry.path());
        let kind = media_type.as_ref().map(FileKind::from).unwrap_or_default();
        let preview_url = if kind == FileKind::Image {
            thumb_urls.map(|b| b.url_for(Path::new(&path)))
        } else {
            None
        };
        factory.set_file(&metadata, media_type, kind, preview_url);
    } else if metadata.is_symlink() {
        // todo: validate that the symlink doesn't go outside where we are allowed to go
        // todo: this probably goes to the wrong place
        let target = read_link(dir_entry.path()).await?;
        let target_path = target.as_os_str().to_str().unwrap().to_owned();

        if !target.exists() {
            factory.set_symlink(true, EntryType::Unknown, target_path);
        } else {
            let target_meta = fs::metadata(&target).await?;
            let target_type = if target_meta.is_dir() {
                EntryType::Directory
            } else if target_meta.is_file() {
                EntryType::File
            } else {
                EntryType::Unknown
            };
            factory.set_symlink(false, target_type, target_path);
        }
    } else {
        return Err(anyhow!("unhandled entry type"));
    }

    factory.build()
}

pub async fn build_entries(cfg: &Config, space: &Space, path: &Path) -> Result<Vec<Entry>> {
    // todo: read meta and live in parallel?
    let live_path = space.live();
    let thumb_urls = match img_thumbs::ThumbUrlBuilder::new(cfg, &space.name, path) {
        Ok(b) => Some(b),
        Err(err) => {
            warn!("could not generate thumb token for {:?}: {:#}", path, err);
            None
        }
    };

    let mut entries = read_dir(live_path.join(path)).await?;
    let mut result = vec![];
    while let Some(dir_entry) = entries.next_entry().await? {
        let filename = dir_entry.file_name();
        match build_entry(dir_entry, &live_path, thumb_urls.as_ref()).await {
            Ok(e) => result.push(e),
            Err(err) => {
                // todo: send errored entries with the list
                error!("couldn't process file: {:?} {:#}", filename, err);
            }
        }
    }

    result.sort_by(|a, b| {
        let a_key = match a.entry_type {
            EntryType::Directory => format!("0_{}", a.name),
            _ => format!("1_{}", a.name),
        };
        let b_key = match b.entry_type {
            EntryType::Directory => format!("0_{}", b.name),
            _ => format!("1_{}", b.name),
        };
        a_key.cmp(&b_key)
    });

    Ok(result)
}
