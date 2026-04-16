use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserPathParams {
    pub user: String,
}
