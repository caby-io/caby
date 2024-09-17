use std::{io, os::unix::fs::MetadataExt, path::{Path, PathBuf}, time::SystemTime};

use axum::async_trait;
use serde::Serialize;
use tokio::fs::{read_dir, DirEntry, ReadDir};
use tracing::warn;

pub mod pretty;

#[derive(Debug, Serialize)]
pub struct Directory {
    pub name: String,
    // Relative path of the file from the mount root
    pub path: String,
    pub created_at: Option<SystemTime>,
    pub pretty_created_at: String,
    pub modified_at: Option<SystemTime>,
    pub pretty_modified_at: String,
}

impl Directory {
    async fn new_from_entry(entry: DirEntry) -> io::Result<Self> {
        let (created_at, modified_at) = entry.metadata().await.map_or((None, None), |meta| {
            (meta.created().ok(), meta.modified().ok())
        });

        Ok(Self {
            name: entry.file_name().into_string().clone().unwrap(),
            path: entry.path().into_os_string().into_string().clone().unwrap(), // TODO: Replace with relative path
            created_at,
            pretty_created_at: pretty::date(created_at),
            modified_at,
            pretty_modified_at: pretty::date(modified_at),
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
}

impl File {
    async fn new_from_entry(entry: DirEntry) -> io::Result<Self> {
        let (size, created_at, modified_at) = entry.metadata().await.map_or((None, None, None), |meta| {
            (Some(meta.size()), meta.created().ok(), meta.modified().ok())
        });

        Ok(Self {
            name: entry.file_name().into_string().clone().unwrap(),
            size,
            pretty_size: pretty::bytes(size),
            path: entry.path().into_os_string().into_string().clone().unwrap(), // TODO: Replace with relative path
            created_at,
            pretty_created_at: pretty::date(created_at),
            modified_at,
            pretty_modified_at: pretty::date(modified_at),
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

pub async fn get_entries(path: &Path) -> io::Result<(Vec<Directory>, Vec<File>)> {
    let full_path = Path::new("/").join(path);
    let mut entries = read_dir(path).await?;

    let mut dirs = vec![];
    let mut files = vec![];

    while let Some(entry) = entries.next_entry().await? {
        let entry_type = if entry.metadata().await?.is_dir() {
            // let name = entry.file_name().into_string().unwrap();
            let dir = Directory::new_from_entry(entry).await.unwrap();
            dirs.push(dir);
        } else {
            files.push(File::new_from_entry(entry).await?);
        };
    }

    Ok((dirs, files))
}
