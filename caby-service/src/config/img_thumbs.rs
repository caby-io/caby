use crate::{config::config_file::ConfigFileImgThumbs, Result};
use anyhow::anyhow;
use std::env::var;

// env vars
pub const ENV_IMG_THUMBS_MAX_EDGE: &str = "CABY_IMG_THUMBS_MAX_EDGE";

// defaults
const DEFAULT_MAX_EDGE: u32 = 256;

#[derive(Clone)]
pub struct ImgThumbsConfig {
    pub max_edge: u32,
}

impl Default for ImgThumbsConfig {
    fn default() -> Self {
        Self {
            max_edge: DEFAULT_MAX_EDGE,
        }
    }
}

impl ImgThumbsConfig {
    pub fn try_new(file: Option<ConfigFileImgThumbs>) -> Result<Self> {
        let file_max_edge = file.and_then(|f| f.max_edge);

        let max_edge = match var(ENV_IMG_THUMBS_MAX_EDGE).ok() {
            Some(raw) => raw.parse::<u32>().map_err(|err| {
                anyhow!(err).context(format!(
                    "{} must be a positive integer",
                    ENV_IMG_THUMBS_MAX_EDGE
                ))
            })?,
            None => file_max_edge.unwrap_or(DEFAULT_MAX_EDGE),
        };

        if max_edge == 0 {
            return Err(anyhow!("img_thumbs.max_edge must be greater than 0"));
        }

        Ok(Self { max_edge })
    }
}
