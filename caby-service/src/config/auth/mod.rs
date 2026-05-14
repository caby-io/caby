use nest_struct::nest_struct;

// env vars
pub const ENV_OIDC_CLIENT_ID: &'static str = "OIDC_CLIENT_ID";
pub const ENV_OIDC_CLIENT_SECRET: &'static str = "OIDC_CLIENT_SECRET";
pub const ENV_OIDC_REDIRECT_URI: &'static str = "OIDC_REDIRECT_URI";
pub const ENV_OIDC_POST_LOGIN_REDIRECT: &'static str = "OIDC_POST_LOGIN_REDIRECT";
pub const ENV_OIDC_SCOPES: &'static str = "OIDC_SCOPES";

pub const ENV_OIDC_ISSUER_URL: &'static str = "OIDC_ISSUER_URL";
pub const ENV_OIDC_AUTHORIZATION_ENDPOINT: &'static str = "OIDC_AUTHORIZATION_ENDPOINT";
pub const ENV_OIDC_TOKEN_ENDPOINT: &'static str = "OIDC_TOKEN_ENDPOINT";
pub const ENV_OIDC_JWKS_URI: &'static str = "OIDC_JWKS_URI";
pub const ENV_OIDC_USERINFO_ENDPOINT: &'static str = "OIDC_USERINFO_ENDPOINT";

// defaults
pub const OIDC_SCOPES_DEFAULT: &[&'static str] = &["openid", "profile", "email"];

#[derive(Clone)]
#[nest_struct]
pub struct AuthConfig {
    pub passwords: nest! {
        #[derive(Clone)]
        pub struct PasswordsAuthConfig {
            pub enabled: bool,
        }
    },
    pub oidc: nest! {
        #[derive(Clone)]
        pub struct OIDCConfig {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub post_login_redirect: String,
            pub scopes: Vec<String>,
            pub provider: nest! {
                #[derive(Clone)]
                enum OidcProviderConfig {
                    Discovery {
                        issuer_url: String,
                    },
                    Manual {
                        issuer_url: String,
                        authorization_endpoint: String,
                        token_endpoint: String,
                        jwks_uri: String,
                        userinfo_endpoint: Option<String>,
                    },
                }
            },
        }
    },
}
