use crate::{
    config::config_file::{ConfigFileAuth, ConfigFileOidc},
    Result,
};
use anyhow::anyhow;
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
    pub oidc: Option<
        nest! {
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
    >,
}

impl Default for AuthConfig {
    fn default() -> Self {
        AuthConfig {
            passwords: PasswordsAuthConfig { enabled: true },
            oidc: None,
        }
    }
}

impl TryFrom<ConfigFileAuth> for AuthConfig {
    type Error = anyhow::Error;

    fn try_from(file: ConfigFileAuth) -> Result<Self> {
        let passwords = PasswordsAuthConfig {
            enabled: file.passwords.and_then(|p| p.enabled).unwrap_or(true),
        };
        let oidc = file.oidc.map(OIDCConfig::try_from).transpose()?;
        Ok(AuthConfig { passwords, oidc })
    }
}

impl TryFrom<ConfigFileOidc> for OIDCConfig {
    type Error = anyhow::Error;

    fn try_from(file: ConfigFileOidc) -> Result<Self> {
        let client_id = file
            .client_id
            .ok_or_else(|| anyhow!(".auth.oidc.client_id is required"))?;
        let client_secret = file
            .client_secret
            .ok_or_else(|| anyhow!(".auth.oidc.client_secret is required"))?;
        let redirect_uri = file
            .redirect_uri
            .ok_or_else(|| anyhow!(".auth.oidc.redirect_uri is required"))?;
        let post_login_redirect = file
            .post_login_redirect
            .ok_or_else(|| anyhow!(".auth.oidc.post_login_redirect is required"))?;
        let scopes = file
            .scopes
            .unwrap_or_else(|| OIDC_SCOPES_DEFAULT.iter().map(|s| s.to_string()).collect());
        let provider = resolve_oidc_provider_config(
            file.issuer_url,
            file.authorization_endpoint,
            file.token_endpoint,
            file.jwks_uri,
            file.userinfo_endpoint,
        )?;

        Ok(OIDCConfig {
            client_id,
            client_secret,
            redirect_uri,
            post_login_redirect,
            scopes,
            provider,
        })
    }
}

fn resolve_oidc_provider_config(
    issuer_url: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    jwks_uri: Option<String>,
    userinfo_endpoint: Option<String>,
) -> Result<OidcProviderConfig> {
    let issuer_url = issuer_url.ok_or_else(|| anyhow!(".auth.oidc.issuer_url is required"))?;

    let any_manual_set = authorization_endpoint.is_some()
        || token_endpoint.is_some()
        || jwks_uri.is_some()
        || userinfo_endpoint.is_some();

    if !any_manual_set {
        return Ok(OidcProviderConfig::Discovery { issuer_url });
    }

    let authorization_endpoint = authorization_endpoint.ok_or_else(|| {
        anyhow!(".auth.oidc.authorization_endpoint is required when manual OIDC endpoints are set")
    })?;
    let token_endpoint = token_endpoint.ok_or_else(|| {
        anyhow!(".auth.oidc.token_endpoint is required when manual OIDC endpoints are set")
    })?;
    let jwks_uri = jwks_uri.ok_or_else(|| {
        anyhow!(".auth.oidc.jwks_uri is required when manual OIDC endpoints are set")
    })?;

    Ok(OidcProviderConfig::Manual {
        issuer_url,
        authorization_endpoint,
        token_endpoint,
        jwks_uri,
        userinfo_endpoint,
    })
}
