use serde::Deserialize;

// todo: move some of these fn's out of the web dir

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
