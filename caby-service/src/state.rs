use axum::extract::FromRef;
use std::sync::Arc;

use crate::{auth::oidc::OidcClient, config::Config, Result};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub oidc_client: Option<Arc<OidcClient>>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let oidc_client = match &config.auth.oidc {
            Some(oidc_cfg) => Some(Arc::new(OidcClient::new(oidc_cfg).await?)),
            None => None,
        };
        Ok(Self {
            config,
            oidc_client,
        })
    }
}

// todo: switch Config to Arc<Config> to save on clone cost
impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Option<Arc<OidcClient>> {
    fn from_ref(state: &AppState) -> Self {
        state.oidc_client.clone()
    }
}
