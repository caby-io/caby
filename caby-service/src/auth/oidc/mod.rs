use anyhow::Context;
use openidconnect::{
    core::{
        CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseType, CoreSubjectIdentifierType,
    },
    AuthUrl, ClientId, ClientSecret, EmptyAdditionalProviderMetadata, IssuerUrl, JsonWebKeySetUrl,
    RedirectUrl, ResponseTypes, TokenUrl, UserInfoUrl,
};
use reqwest::Client as HttpClient;
use tracing::info;

use crate::{
    config::auth::{OIDCConfig, OidcProviderConfig},
    Result,
};

pub struct OidcClient {
    pub http: HttpClient,
    pub metadata: CoreProviderMetadata,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub redirect_uri: RedirectUrl,
}

impl OidcClient {
    pub async fn new(cfg: &OIDCConfig) -> Result<Self> {
        let http = HttpClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("could not build OIDC HTTP client")?;

        let issuer_url_str = match &cfg.provider {
            OidcProviderConfig::Discovery { issuer_url } => issuer_url.clone(),
            OidcProviderConfig::Manual { issuer_url, .. } => issuer_url.clone(),
        };

        let metadata = match &cfg.provider {
            OidcProviderConfig::Discovery { issuer_url } => {
                let issuer =
                    IssuerUrl::new(issuer_url.clone()).context("invalid OIDC issuer_url")?;
                CoreProviderMetadata::discover_async(issuer, &http)
                    .await
                    .with_context(|| format!("OIDC discovery failed for {}", issuer_url))?
            }
            OidcProviderConfig::Manual {
                issuer_url,
                authorization_endpoint,
                token_endpoint,
                jwks_uri,
                userinfo_endpoint,
            } => {
                let issuer =
                    IssuerUrl::new(issuer_url.clone()).context("invalid OIDC issuer_url")?;

                let auth_url = AuthUrl::new(authorization_endpoint.clone())
                    .context("invalid OIDC authorization_endpoint")?;

                let jwks_url =
                    JsonWebKeySetUrl::new(jwks_uri.clone()).context("invalid OIDC jwks_uri")?;

                let mut metadata = CoreProviderMetadata::new(
                    issuer,
                    auth_url,
                    jwks_url,
                    vec![ResponseTypes::new(vec![CoreResponseType::Code])],
                    vec![CoreSubjectIdentifierType::Public],
                    vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256],
                    EmptyAdditionalProviderMetadata {},
                );

                metadata = metadata.set_token_endpoint(Some(
                    TokenUrl::new(token_endpoint.clone()).context("invalid OIDC token_endpoint")?,
                ));

                if let Some(userinfo) = userinfo_endpoint {
                    metadata = metadata.set_userinfo_endpoint(Some(
                        UserInfoUrl::new(userinfo.clone())
                            .context("invalid OIDC userinfo_endpoint")?,
                    ));
                }

                metadata
            }
        };

        let client_id = ClientId::new(cfg.client_id.clone());
        let client_secret = ClientSecret::new(cfg.client_secret.clone());
        let redirect_uri =
            RedirectUrl::new(cfg.redirect_uri.clone()).context("invalid OIDC redirect_uri")?;

        info!("oidc: discovery succeeded for {}", issuer_url_str);

        Ok(Self {
            http,
            metadata,
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}
