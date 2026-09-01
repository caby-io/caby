use serde::Deserialize;

pub mod shares_admin;
pub mod shares_auth;
pub mod shares_create;
pub mod shares_delete;
pub mod shares_download;
pub mod shares_get;
pub mod shares_list;
pub mod shares_list_files;

#[derive(Deserialize)]
pub struct ShareIdParam {
    pub id: String,
}

pub use shares_admin::handle_admin_get_share;
pub use shares_auth::handle_password_auth_share;
pub use shares_create::handle_create_share;
pub use shares_delete::handle_delete_share;
pub use shares_download::handle_download_share;
pub use shares_get::handle_get_share;
pub use shares_list::handle_list_shares;
pub use shares_list_files::handle_list_share_files;
