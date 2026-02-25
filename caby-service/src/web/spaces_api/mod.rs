use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{config::Config, jsend::JSendBuilder, space::Space};

#[derive(Serialize)]
pub struct SpaceResponse {
    pub name: String,
}

impl From<Space> for SpaceResponse {
    fn from(value: Space) -> Self {
        Self {
            name: value.name.clone(),
        }
    }
}

// todo: obfuscate the path from both endpoints?
pub async fn handle_list_spaces(State(cfg): State<Config>) -> Response {
    let spaces: Vec<SpaceResponse> = cfg
        .spaces
        .values()
        .map(|s| {
            let space: Space = s.clone().into();
            SpaceResponse::from(space)
        })
        .collect();

    JSendBuilder::new().success(spaces).into_response()
}

pub async fn handle_show_space(State(cfg): State<Config>, space: Space) -> Response {
    JSendBuilder::new()
        .success(SpaceResponse::from(space))
        .into_response()
}
