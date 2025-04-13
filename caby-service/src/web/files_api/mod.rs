pub mod files_delete;
pub mod files_download;
pub mod files_list;
pub mod files_move;
pub mod files_put;
pub mod files_upload;

// Re-export for cleanliness
pub use files_delete::handle_delete_files;
pub use files_download::handle_download_files;
pub use files_list::handle_list_files;
pub use files_move::handle_move_files;
pub use files_put::handle_put_files;
pub use files_upload::handle_register_upload;

// todo: TEMPORARY Move me to config
pub static ROOT_PATH: &str = "/home/suhaib/caby-home";
pub static TMP_PATH: &str = "/home/suhaib/caby-home/.tmp";
