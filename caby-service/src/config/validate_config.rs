use crate::error::Result;
use anyhow::anyhow;

// Meta filenames must:
// 1. Must be UTF-8 (already done if we have a &str)
// 2. Cannot be empty
// 3. Cannot contain any slashes
pub fn is_valid_meta_filename(filename: &str) -> Result<()> {
    if filename.is_empty() {
        return Err(anyhow!("filename cannot be empty"));
    }

    if filename.contains("/") {
        return Err(anyhow!("filename cannot contain slashes"));
    }

    if filename.len() > 255 {
        return Err(anyhow!("filename must be less than 255 characters"));
    }

    Ok(())
}
