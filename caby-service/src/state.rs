use axum::extract::FromRef;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
}

// todo: switch Config to Arc<Config> to save on clone cost
impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
