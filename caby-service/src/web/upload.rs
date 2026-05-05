use serde::Deserialize;

use crate::upload::manifest::{ManifestEntryType, UploadManifestEntry};

pub static MAX_CHUNK_SIZE: u64 = 10_000_000;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum UploadEntryType {
    Directory,
    File,
}

// todo: need to consider how best to handle empty dirs
#[derive(Deserialize, Debug)]
pub struct UploadEntry {
    pub entry_type: UploadEntryType,
    pub name: String,
    pub size: Option<u64>,
    pub xxh_digest: Option<String>,
}

impl From<UploadEntry> for UploadManifestEntry {
    fn from(val: UploadEntry) -> Self {
        Self {
            entry_type: match val.entry_type {
                UploadEntryType::Directory => ManifestEntryType::Directory,
                UploadEntryType::File => ManifestEntryType::File,
            },
            name: val.name,
            size: val.size,
            xxh_digest: val.xxh_digest,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    Override,
    Skip,
    Prompt,
    Deconflict,
}

// Upload tokens

pub const HEADER_UPLOAD_TOKEN: &str = "Caby-Upload-Token";
pub const HEADER_CHUNK_INDEX: &str = "Caby-Chunk-Index";
