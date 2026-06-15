use crate::{config::config_file::ConfigFileUrls, Result};
use anyhow::{anyhow, Context};
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
}

impl UrlsConfig {
    pub fn cors_allowed_origins(&self) -> Result<Vec<HeaderValue>> {
        let mut origins = vec![origin_header_from_url(".urls.frontend", &self.frontend)?];

        if let Ok(raw) = var(ENV_CORS_EXTRA_ORIGINS) {
            for entry in raw.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                let url = Url::parse(entry).with_context(|| {
                    format!(
                        "{} contains an invalid URL: {:?}",
                        ENV_CORS_EXTRA_ORIGINS, entry
                    )
                })?;
                let label = format!("{} entry {:?}", ENV_CORS_EXTRA_ORIGINS, entry);
                origins.push(origin_header_from_url(&label, &url)?);
            }
        }

        Ok(origins)
    }
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

impl TryFrom<Option<ConfigFileUrls>> for UrlsConfig {
    type Error = anyhow::Error;

    fn try_from(file: Option<ConfigFileUrls>) -> Result<Self> {
        let (file_backend, file_frontend) = match file {
            Some(f) => (f.backend, f.frontend),
            None => (None, None),
        };

        let backend = resolve_url(
            "backend",
            ".urls.backend",
            ENV_BACKEND_URL,
            var(ENV_BACKEND_URL).ok().or(file_backend),
        )?;
        let frontend = resolve_url(
            "frontend",
            ".urls.frontend",
            ENV_FRONTEND_URL,
            var(ENV_FRONTEND_URL).ok().or(file_frontend),
        )?;

        Ok(UrlsConfig { backend, frontend })
    }
}

fn resolve_url(field: &str, yaml_path: &str, env_name: &str, raw: Option<String>) -> Result<Url> {
    let raw =
        raw.ok_or_else(|| anyhow!("{} url is required ({} or {})", field, yaml_path, env_name))?;

    let mut url =
        Url::parse(&raw).with_context(|| format!("{} is not a valid URL: {:?}", yaml_path, raw))?;

    if url.query().is_some() || url.fragment().is_some() {
        return Err(anyhow!(
            "{} must not include a query or fragment",
            yaml_path
        ));
    }

    if !url.path().ends_with('/') {
        let normalized = format!("{}/", url.path());
        url.set_path(&normalized);
    }

    Ok(url)
}
