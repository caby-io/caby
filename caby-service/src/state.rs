use axum::extract::FromRef;
use std::sync::Arc;

use crate::{
    auth::oidc::OidcClient,
    config::Config,
    controller::{Controller, PathLocks},
    event::Sender,
    Result,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub oidc_client: Option<Arc<OidcClient>>,
    pub controller: Arc<Controller>,
    pub events_tx: Sender,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let oidc_client = match &config.auth.oidc {
            Some(oidc_cfg) => Some(Arc::new(OidcClient::new(oidc_cfg).await?)),
            None => None,
        };
        let (controller, events_tx) = Controller::new(config.clone());
        Ok(Self {
            config,
            oidc_client,
            controller,
            events_tx,
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

impl FromRef<AppState> for Arc<Controller> {
    fn from_ref(state: &AppState) -> Self {
        state.controller.clone()
    }
}

impl FromRef<AppState> for Arc<PathLocks> {
    fn from_ref(state: &AppState) -> Self {
        state.controller.locks()
    }
}

impl FromRef<AppState> for Sender {
    fn from_ref(state: &AppState) -> Self {
        state.events_tx.clone()
    }
}
