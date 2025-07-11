use std::{
    default,
    io::{self, ErrorKind},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

use path_clean::PathClean;
use serde::Serialize;
use serde_json::error;
use tokio::fs::{self, metadata, read_dir, read_link, DirEntry, ReadDir};
use tracing::{debug, error, warn};

pub mod pretty;
pub mod upload;

#[derive(Serialize, Default)]
// #[strum(serialize_all = "snake_case")]
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
    pub path: String, // relative path of the file from the mount root
    pub created_at: Option<SystemTime>,
    pub pretty_created_at: String,
    pub modified_at: Option<SystemTime>,
    pub pretty_modified_at: String,

    // extra fields
    pub entry_fields: Option<EntryFields>,
}

// impl Entry {
//     // todo: make safe
//     // todo: handle symlinks
//     pub fn set_path(&mut self, root_path: &PathBuf) {
//         self.path = root_path
//             .join(&self.name)
//             .as_os_str()
//             .to_str()
//             .unwrap()
//             .to_owned();
//     }
// }

impl Entry {
    async fn try_from(root_path: &Path, value: DirEntry) -> Result<Self, io::Error> {
        let metadata = value.metadata().await?;

        value.path().strip_prefix(root_path);

        // Fill common fields
        let created_at = metadata.created().ok();
        let pretty_created_at = pretty::date(&created_at);
        let modified_at = metadata.modified().ok();
        let pretty_modified_at = pretty::date(&modified_at);

        let mut entry = Self {
            entry_type: EntryType::default(),
            name: value
                .file_name()
                .into_string()
                .map_err(|err| io::Error::new(ErrorKind::Other, "couldn't convert to string"))?,
            // path: root_path
            //     .join(value.file_name())
            //     .to_str()
            //     .ok_or(io::Error::new(
            //         ErrorKind::Other,
            //         "couldn't convert root path to string",
            //     ))?
            //     .to_owned(),
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
            created_at,
            pretty_created_at,
            modified_at,
            pretty_modified_at,
            entry_fields: None,
        };

        if metadata.is_dir() {
            entry.entry_type = EntryType::Directory;
            return Ok(entry);
        }

        if metadata.is_file() {
            let size = metadata.size();
            entry.entry_type = EntryType::File;
            entry.entry_fields = Some(EntryFields::File {
                size,
                pretty_size: pretty::bytes(size),
            });
            return Ok(entry);
        }

        if metadata.is_symlink() {
            entry.entry_type = EntryType::Symlink;

            // todo: validate that the symlink doesn't go outside where we are allowed to go
            // todo: this probably goes to the wrong place
            let target = read_link(value.path()).await?;
            let target_path = target.as_os_str().to_str().unwrap().to_owned();

            if !target.exists() {
                entry.entry_fields = Some(EntryFields::Symlink {
                    is_broken: true,
                    target_type: EntryType::Unknown,
                    target_path,
                });
                return Ok(entry);
            }

            let metadata = fs::metadata(target).await?;
            if metadata.is_dir() {
                entry.entry_fields = Some(EntryFields::Symlink {
                    is_broken: false,
                    target_type: EntryType::Directory,
                    target_path,
                });
                return Ok(entry);
            }

            if metadata.is_file() {
                entry.entry_fields = Some(EntryFields::Symlink {
                    is_broken: false,
                    target_type: EntryType::File,
                    target_path,
                });
                return Ok(entry);
            }

            return Ok(entry);
        }

        return Err(io::Error::new(ErrorKind::Other, "unhandled entry type"));
    }
}

// Returns a sanitized full path from the input path
pub fn joined_path(root_path: &Path, relative_path: &Path) -> Option<PathBuf> {
    let path = root_path.join(relative_path.clean()).clean();
    if path.starts_with(root_path) {
        return Some(path);
    }
    None
}

pub async fn build_entries(root_path: &Path, path: &Path) -> io::Result<Vec<Entry>> {
    let full_path = Path::new("/").join(path);
    let mut entries = read_dir(path).await?;

    let mut result = vec![];

    while let Some(dir_entry) = entries.next_entry().await? {
        let filename = dir_entry.file_name();
        match Entry::try_from(root_path, dir_entry).await {
            Ok(e) => result.push(e),
            Err(err) => {
                error!("couldn't process file: {:?} {}", filename, err);
            }
        }
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
