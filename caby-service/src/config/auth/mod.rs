use crate::{
    config::{config_file::ConfigFileAuth, urls::UrlsConfig},
    Result,
};
use anyhow::anyhow;
use nest_struct::nest_struct;
use std::{collections::BTreeSet, env::var};

// env vars
pub const ENV_OIDC_CLIENT_ID: &'static str = "OIDC_CLIENT_ID";
pub const ENV_OIDC_CLIENT_SECRET: &'static str = "OIDC_CLIENT_SECRET";
pub const ENV_OIDC_REDIRECT_URI: &'static str = "OIDC_REDIRECT_URI";
pub const ENV_OIDC_POST_LOGIN_REDIRECT: &'static str = "OIDC_POST_LOGIN_REDIRECT";
pub const ENV_OIDC_EXTRA_SCOPES: &'static str = "OIDC_EXTRA_SCOPES";

pub const ENV_OIDC_ISSUER_URL: &'static str = "OIDC_ISSUER_URL";
pub const ENV_OIDC_AUTHORIZATION_ENDPOINT: &'static str = "OIDC_AUTHORIZATION_ENDPOINT";
pub const ENV_OIDC_TOKEN_ENDPOINT: &'static str = "OIDC_TOKEN_ENDPOINT";
pub const ENV_OIDC_JWKS_URI: &'static str = "OIDC_JWKS_URI";
pub const ENV_OIDC_USERINFO_ENDPOINT: &'static str = "OIDC_USERINFO_ENDPOINT";

// defaults
pub const DEFAULT_OIDC_SCOPES: &[&'static str] = &["profile", "email"];

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
                // using btree so that we have a consistent list between requests
                pub scopes: BTreeSet<String>,
                pub provider: nest! {
                    #[derive(Clone)]
                    pub enum OidcProviderConfig {
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

fn parse_scopes(raw: String) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl AuthConfig {
    pub fn try_new(file_part: Option<ConfigFileAuth>, urls: &UrlsConfig) -> Result<Self> {
        let mut b = AuthConfigBuilder::new();

        // url-derived defaults
        b.try_set_oidc_redirect_uri(Some(format!(
            "{}v0/auth/oidc/callback",
            urls.backend.as_str()
        )));
        b.try_set_oidc_post_login_redirect(Some(format!(
            "{}login/oidc/callback",
            urls.frontend.as_str()
        )));

        // config file
        if let Some(f) = file_part {
            b.try_set_passwords_enabled(f.passwords.and_then(|p| p.enabled));

            if let Some(o) = f.oidc {
                b.try_set_oidc_client_id(o.client_id)
                    .try_set_oidc_client_secret(o.client_secret)
                    .try_set_oidc_redirect_uri(o.redirect_uri)
                    .try_set_oidc_post_login_redirect(o.post_login_redirect)
                    .try_set_oidc_issuer_url(o.issuer_url)
                    .try_set_oidc_authorization_endpoint(o.authorization_endpoint)
                    .try_set_oidc_token_endpoint(o.token_endpoint)
                    .try_set_oidc_jwks_uri(o.jwks_uri)
                    .try_set_oidc_userinfo_endpoint(o.userinfo_endpoint);
            };
        };

        // env overrides
        b.try_set_oidc_client_id(var(ENV_OIDC_CLIENT_ID).ok())
            .try_set_oidc_client_secret(var(ENV_OIDC_CLIENT_SECRET).ok())
            .try_set_oidc_redirect_uri(var(ENV_OIDC_REDIRECT_URI).ok())
            .try_set_oidc_post_login_redirect(var(ENV_OIDC_POST_LOGIN_REDIRECT).ok())
            .try_add_oidc_scopes(var(ENV_OIDC_EXTRA_SCOPES).ok().map(parse_scopes))
            .try_set_oidc_issuer_url(var(ENV_OIDC_ISSUER_URL).ok())
            .try_set_oidc_authorization_endpoint(var(ENV_OIDC_AUTHORIZATION_ENDPOINT).ok())
            .try_set_oidc_token_endpoint(var(ENV_OIDC_TOKEN_ENDPOINT).ok())
            .try_set_oidc_jwks_uri(var(ENV_OIDC_JWKS_URI).ok())
            .try_set_oidc_userinfo_endpoint(var(ENV_OIDC_USERINFO_ENDPOINT).ok());

        b.build()
    }
}

fn build_oidc_provider(
    issuer_url: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    jwks_uri: Option<String>,
    userinfo_endpoint: Option<String>,
) -> Result<OidcProviderConfig> {
    let issuer_url = issuer_url.ok_or_else(|| {
        anyhow!(
            "OIDC issuer_url is required (.auth.oidc.issuer_url or {})",
            ENV_OIDC_ISSUER_URL
        )
    })?;

    if authorization_endpoint.is_none()
        && token_endpoint.is_none()
        && jwks_uri.is_none()
        && userinfo_endpoint.is_none()
    {
        return Ok(OidcProviderConfig::Discovery { issuer_url });
    }

    let authorization_endpoint = authorization_endpoint.ok_or_else(|| {
            anyhow!(
                "OIDC authorization_endpoint is required for manual provider discovery (.auth.oidc.authorization_endpoint or {})",
                ENV_OIDC_AUTHORIZATION_ENDPOINT
            )
        })?;
    let token_endpoint = token_endpoint.ok_or_else(|| {
            anyhow!(
                "OIDC token_endpoint is required for manual provider discovery (.auth.oidc.token_endpoint or {})",
                ENV_OIDC_TOKEN_ENDPOINT
            )
        })?;
    let jwks_uri = jwks_uri.ok_or_else(|| {
        anyhow!(
            "OIDC jwks_uri is required for manual provider discovery (.auth.oidc.jwks_uri or {})",
            ENV_OIDC_JWKS_URI
        )
    })?;

    Ok(OidcProviderConfig::Manual {
        issuer_url,
        authorization_endpoint,
        token_endpoint,
        jwks_uri,
        userinfo_endpoint,
    })
}

// todo: consider making a sub-builder for oidc if passwords expands
#[derive(Default)]
pub struct AuthConfigBuilder {
    passwords_enabled: bool,
    oidc_issuer_url: Option<String>,
    oidc_client_id: Option<String>,
    oidc_client_secret: Option<String>,
    oidc_redirect_uri: Option<String>,
    oidc_post_login_redirect: Option<String>,
    oidc_scopes: BTreeSet<String>,
    oidc_authorization_endpoint: Option<String>,
    oidc_token_endpoint: Option<String>,
    oidc_jwks_uri: Option<String>,
    oidc_userinfo_endpoint: Option<String>,
}

impl AuthConfigBuilder {
    pub fn new() -> Self {
        Self {
            passwords_enabled: true,
            oidc_scopes: DEFAULT_OIDC_SCOPES.iter().map(|s| s.to_string()).collect(),
            ..Self::default()
        }
    }

    pub fn try_set_passwords_enabled(&mut self, v: Option<bool>) -> &mut Self {
        if let Some(v) = v {
            self.passwords_enabled = v;
        }
        self
    }

    pub fn try_set_oidc_client_id(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_client_id = Some(v);
        }
        self
    }

    pub fn try_set_oidc_client_secret(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_client_secret = Some(v);
        }
        self
    }

    pub fn try_set_oidc_redirect_uri(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_redirect_uri = Some(v);
        }
        self
    }

    pub fn try_set_oidc_post_login_redirect(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_post_login_redirect = Some(v);
        }
        self
    }

    pub fn try_add_oidc_scopes(&mut self, v: Option<Vec<String>>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_scopes.extend(v);
        }
        self
    }

    pub fn try_set_oidc_issuer_url(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_issuer_url = Some(v);
        }
        self
    }

    pub fn try_set_oidc_authorization_endpoint(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_authorization_endpoint = Some(v);
        }
        self
    }

    pub fn try_set_oidc_token_endpoint(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_token_endpoint = Some(v);
        }
        self
    }

    pub fn try_set_oidc_jwks_uri(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_jwks_uri = Some(v);
        }
        self
    }

    pub fn try_set_oidc_userinfo_endpoint(&mut self, v: Option<String>) -> &mut Self {
        if let Some(v) = v {
            self.oidc_userinfo_endpoint = Some(v);
        }
        self
    }

    fn build_oidc_config(self) -> Result<OIDCConfig> {
        let client_id = self.oidc_client_id.ok_or_else(|| {
            anyhow!(
                "OIDC client_id is required (.auth.oidc.client_id or {})",
                ENV_OIDC_CLIENT_ID
            )
        })?;
        let client_secret = self.oidc_client_secret.ok_or_else(|| {
            anyhow!(
                "OIDC client_secret is required (.auth.oidc.client_secret or {})",
                ENV_OIDC_CLIENT_SECRET
            )
        })?;

        let redirect_uri = self.oidc_redirect_uri.ok_or_else(|| {
            anyhow!(
                "OIDC redirect_uri is required (.auth.oidc.redirect_uri or {})",
                ENV_OIDC_REDIRECT_URI
            )
        })?;
        url::Url::parse(&redirect_uri)
            .map_err(|err| anyhow!(err).context("OIDC redirect URI must be a valid URL"))?;

        let post_login_redirect = self.oidc_post_login_redirect.ok_or_else(|| {
            anyhow!(
                "OIDC post_login_redirect is required (.auth.oidc.post_login_redirect or {})",
                ENV_OIDC_POST_LOGIN_REDIRECT
            )
        })?;
        url::Url::parse(&post_login_redirect).map_err(|err| {
            anyhow!(err).context(".auth.oidc.post_login_redirect must be a valid URL")
        })?;

        let provider = build_oidc_provider(
            self.oidc_issuer_url,
            self.oidc_authorization_endpoint,
            self.oidc_token_endpoint,
            self.oidc_jwks_uri,
            self.oidc_userinfo_endpoint,
        )?;

        Ok(OIDCConfig {
            client_id,
            client_secret,
            redirect_uri,
            post_login_redirect,
            scopes: self.oidc_scopes,
            provider,
        })
    }

    pub fn build(self) -> Result<AuthConfig> {
        let passwords = PasswordsAuthConfig {
            enabled: self.passwords_enabled,
        };

        let oidc = if self.oidc_issuer_url.is_some() {
            Some(self.build_oidc_config()?)
        } else {
            // todo: warn the user if other oidc fields are set
            None
        };

        Ok(AuthConfig { passwords, oidc })
    }
}
