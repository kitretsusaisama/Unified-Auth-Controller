use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OidcProviderMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
}

impl Default for OidcProviderMetadata {
    fn default() -> Self {
        Self {
            issuer: "http://localhost:8080".to_string(),
            authorization_endpoint: "http://localhost:8080/auth/authorize".to_string(),
            token_endpoint: "http://localhost:8080/auth/token".to_string(),
            userinfo_endpoint: "http://localhost:8080/auth/userinfo".to_string(),
            jwks_uri: "http://localhost:8080/auth/certs".to_string(),
            scopes_supported: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
            ],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        }
    }
}

pub fn generate_oidc_metadata(base_url: &str) -> OidcProviderMetadata {
    OidcProviderMetadata {
        issuer: base_url.to_string(),
        authorization_endpoint: format!("{}/auth/authorize", base_url),
        token_endpoint: format!("{}/auth/token", base_url),
        userinfo_endpoint: format!("{}/auth/userinfo", base_url),
        jwks_uri: format!("{}/auth/certs", base_url),
        ..Default::default()
    }
}
