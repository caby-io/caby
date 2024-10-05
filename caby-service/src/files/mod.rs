use std::{
    io,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::SystemTime,
};

use axum::async_trait;
use serde::Serialize;
use tokio::fs::{self, metadata, read_dir, read_link, DirEntry, ReadDir};
use tracing::warn;

pub mod pretty;

#[derive(Debug, Serialize)]
pub struct Symlink {
    is_broken: bool,
    target_path: String,
}

#[derive(Debug, Serialize)]
pub struct Directory {
    pub name: String,
    // Relative path of the file from the mount root
    pub path: String,
    pub created_at: Option<SystemTime>,
    pub pretty_created_at: String,
    pub modified_at: Option<SystemTime>,
    pub pretty_modified_at: String,
    pub symlink: Option<Symlink>,
}

impl Directory {
    async fn new_from_entry(entry: DirEntry, root_path: &PathBuf) -> io::Result<Self> {
        let (created_at, modified_at) = entry.metadata().await.map_or((None, None), |meta| {
            (meta.created().ok(), meta.modified().ok())
        });

        Ok(Self {
            name: entry.file_name().into_string().clone().unwrap(),
            path: root_path
                .join(entry.file_name())
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned(),
            created_at,
            pretty_created_at: pretty::date(created_at),
            modified_at,
            pretty_modified_at: pretty::date(modified_at),
            symlink: None,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct File {
    pub name: String,
    pub size: Option<u64>,
    pub path: String,
    pub pretty_size: String,
    pub created_at: Option<SystemTime>,
    pub pretty_created_at: String,
    pub modified_at: Option<SystemTime>,
    pub pretty_modified_at: String,
    pub symlink: Option<Symlink>,
}

impl File {
    async fn new_from_entry(entry: DirEntry, root_path: &PathBuf) -> io::Result<Self> {
        let (size, created_at, modified_at) =
            entry.metadata().await.map_or((None, None, None), |meta| {
                (Some(meta.size()), meta.created().ok(), meta.modified().ok())
            });

        Ok(Self {
            name: entry.file_name().into_string().clone().unwrap(),
            size,
            pretty_size: pretty::bytes(size),
            path: root_path
                .join(entry.file_name())
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned(),
            created_at,
            pretty_created_at: pretty::date(created_at),
            modified_at,
            pretty_modified_at: pretty::date(modified_at),
            symlink: None,
        })
    }
}

pub fn sanitize_path(path: &Path) -> Box<Path> {
    let mut buf = PathBuf::new();
    path.components().into_iter().for_each(|c| {
        buf.push(c);
    });
    buf.into_boxed_path()
}

pub async fn get_entries(
    path: &Path,
    root_path: &PathBuf,
) -> io::Result<(Vec<Directory>, Vec<File>)> {
    let full_path = Path::new("/").join(path);
    let mut entries = read_dir(path).await?;

    let mut dirs = vec![];
    let mut files = vec![];

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        if metadata.is_dir() {
            let dir = Directory::new_from_entry(entry, root_path).await.unwrap();
            dirs.push(dir);
            continue;
        }

        if metadata.is_file() {
            files.push(File::new_from_entry(entry, root_path).await?);
            continue;
        }

        if metadata.is_symlink() {
            let path = read_link(entry.path()).await?;
            // todo: ensure that the symlink doesn't lead outside of where we can go.
            // relative target path that we care about
            let target_path = root_path
                .join(path.clone())
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned();

            if !path.exists() {
                warn!("symlink doesn't exist: {:?}", entry.file_name());
                let mut link = File::new_from_entry(entry, root_path).await?;
                link.symlink = Some(Symlink {
                    is_broken: true,
                    target_path,
                });
                files.push(link);
                continue;
            }

            let metadata = fs::metadata(path).await?;
            if metadata.is_dir() {
                let mut dir = Directory::new_from_entry(entry, root_path).await?;
                dir.symlink = Some(Symlink {
                    is_broken: false,
                    target_path,
                });
                dirs.push(dir);
                continue;
            }

            if metadata.is_file() {
                let mut file = File::new_from_entry(entry, root_path).await?;
                file.symlink = Some(Symlink {
                    is_broken: false,
                    target_path,
                });
                files.push(file);
                continue;
            }
        }

        warn!("unhandled file/dir: {:?}", entry.file_name())
    }

    Ok((dirs, files))
}
