pub mod auth_login;
pub mod auth_logout;
pub mod auth_test;

// Re-export for cleanliness
pub use auth_login::handle_login;
pub use auth_logout::handle_logout;
pub use auth_test::handle_test_auth;
