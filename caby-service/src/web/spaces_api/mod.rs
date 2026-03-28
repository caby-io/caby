use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{config::Config, jsend::JSendBuilder, space::Space};

#[derive(Serialize)]
pub struct SpaceResponse<'a> {
    pub name: &'a str,
    pub display: &'a str,
}

impl<'a> From<&'a Space> for SpaceResponse<'a> {
    fn from(value: &'a Space) -> Self {
        Self {
            name: &value.name,
            display: &value.display,
        }
    }
}

// todo: obfuscate the path from both endpoints?
pub async fn handle_list_spaces(State(cfg): State<Config>) -> Response {
    let spaces: Vec<Space> = cfg.spaces.values().map(|s| s.clone().into()).collect();
    let spaces: Vec<SpaceResponse> = spaces.iter().map(SpaceResponse::from).collect();

    JSendBuilder::new().success(spaces).into_response()
}

pub async fn handle_show_space(State(cfg): State<Config>, space: Space) -> Response {
    JSendBuilder::new()
        .success(SpaceResponse::from(&space))
        .into_response()
}
