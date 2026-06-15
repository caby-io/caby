use crate::{config::config_file::ConfigFileUrls, Result};
use anyhow::anyhow;
use axum::http::HeaderValue;
use std::env::var;
use url::Url;

pub const ENV_BACKEND_URL: &str = "CABY_BACKEND_URL";
pub const ENV_FRONTEND_URL: &str = "CABY_FRONTEND_URL";
pub const ENV_CORS_EXTRA_ORIGINS: &str = "CORS_EXTRA_ORIGINS";

#[derive(Clone)]
pub struct UrlsConfig {
    pub backend: Url,
    pub frontend: Url,
    pub cors_allowed_origins: Vec<HeaderValue>,
}

fn origin_header_from_url(label: &str, url: &Url) -> Result<HeaderValue> {
    let origin = url.origin().ascii_serialization();
    if origin == "null" {
        return Err(anyhow!(
            "{} has an opaque origin and cannot be used for CORS",
            label
        ));
    }
    HeaderValue::from_str(&origin)
        .map_err(|e| anyhow!(e).context(format!("{} origin is not a valid HeaderValue", label)))
}

impl UrlsConfig {
    pub fn try_new(file: Option<ConfigFileUrls>) -> Result<Self> {
        let (file_backend, file_frontend) = match file {
            Some(f) => (f.backend, f.frontend),
            None => (None, None),
        };

        let input_backend = var(ENV_BACKEND_URL).ok().or(file_backend).ok_or_else(|| {
            anyhow!(
                "Backend URL is required (.urls.backend or {})",
                ENV_BACKEND_URL
            )
        })?;
        let backend = Url::parse(&input_backend)
            .map_err(|err| anyhow!(err).context("Backend URL must be a valid URL"))?;

        let input_frontend = var(ENV_FRONTEND_URL)
            .ok()
            .or(file_frontend)
            .ok_or_else(|| {
                anyhow!(
                    "Frontend URL is required (.urls.frontend or {})",
                    ENV_FRONTEND_URL
                )
            })?;
        let frontend = Url::parse(&input_frontend)
            .map_err(|err| anyhow!(err).context("Frontend URL must be a valid URL"))?;

        let mut cors_allowed_origins = vec![origin_header_from_url("Frontend URL", &frontend)?];
        if let Ok(raw) = var(ENV_CORS_EXTRA_ORIGINS) {
            for entry in raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                let label = format!("{} entry {:?}", ENV_CORS_EXTRA_ORIGINS, entry);
                let url = Url::parse(entry).map_err(|err| {
                    anyhow!(err).context(format!("{} must be a valid URL", label))
                })?;
                cors_allowed_origins.push(origin_header_from_url(&label, &url)?);
            }
        }

        Ok(UrlsConfig {
            backend,
            frontend,
            cors_allowed_origins,
        })
    }
}
