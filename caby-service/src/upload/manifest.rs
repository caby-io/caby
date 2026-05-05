use std::path::Path;

use anyhow::anyhow;
use bitcode::{Decode, Encode};
use tokio::fs;

use crate::Result;

#[derive(Encode, Decode)]
pub struct UploadManifest {
    pub token: String,
    pub entries: Vec<UploadManifestEntry>,
}

#[derive(Encode, Decode)]
pub struct UploadManifestEntry {
    pub entry_type: ManifestEntryType,
    pub name: String,
    pub size: Option<u64>,
    pub xxh_digest: Option<String>,
}

#[derive(Encode, Decode, PartialEq)]
pub enum ManifestEntryType {
    Directory,
    File,
}

pub async fn write(upload_dir: &Path, manifest: &UploadManifest) -> Result<()> {
    let path = upload_dir.join("manifest");
    let bytes = bitcode::encode(manifest);
    fs::write(path, &bytes)
        .await
        .map_err(|err| anyhow!(err).context("could not write upload manifest"))
}

pub async fn read(upload_dir: &Path) -> Result<UploadManifest> {
    let path = upload_dir.join("manifest");
    let bytes = fs::read(path)
        .await
        .map_err(|err| anyhow!(err).context("could not read upload manifest"))?;
    bitcode::decode(&bytes).map_err(|err| anyhow!(err).context("could not decode upload manifest"))
}
