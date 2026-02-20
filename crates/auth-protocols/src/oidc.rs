use anyhow::{Context, Result};
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType, CoreJsonWebKeySet};
use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, RedirectUrl, Scope,
    AuthenticationFlow, CsrfToken, Nonce, AuthUrl, TokenUrl,
};
use openidconnect::reqwest::async_http_client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OidcService {
    client: CoreClient,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

impl OidcService {
    pub async fn new(config: OidcConfig) -> Result<Self> {
        let issuer_url = IssuerUrl::new(config.issuer_url.clone())?;

        // Discover the provider metadata
        let provider_metadata = CoreProviderMetadata::discover_async(
            issuer_url,
            async_http_client,
        )
        .await
        .context("Failed to discover OpenID Provider metadata")?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url)?);

        Ok(Self { client })
    }

    pub fn new_manual(config: OidcConfig, auth_url: String, token_url: String) -> Result<Self> {
        let client = CoreClient::new(
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
            IssuerUrl::new(config.issuer_url)?,
            AuthUrl::new(auth_url)?,
            Some(TokenUrl::new(token_url)?),
            None,
            CoreJsonWebKeySet::default(),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_url)?);

        Ok(Self { client })
    }

    pub fn get_authorization_url(&self) -> (String, CsrfToken, Nonce) {
        let (auth_url, csrf_token, nonce) = self.client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        (auth_url.to_string(), csrf_token, nonce)
    }

    // Additional methods for token exchange would go here
}