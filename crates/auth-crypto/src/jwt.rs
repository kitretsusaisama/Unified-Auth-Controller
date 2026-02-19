//! JWT token operations with RS256 support

use crate::keys::KeyManager;
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token encoding failed: {0}")]
    EncodingError(#[from] jsonwebtoken::errors::Error),
    #[error("Token validation failed: {reason}")]
    ValidationError { reason: String },
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Key management error: {0}")]
    KeyError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,      // Subject (user ID)
    pub iss: String,      // Issuer
    pub aud: String,      // Audience
    pub exp: i64,         // Expiration time
    pub iat: i64,         // Issued at
    pub nbf: i64,         // Not before
    pub jti: String,      // JWT ID
    pub tenant_id: String, // Tenant ID for multi-tenancy
    pub permissions: Vec<String>, // User permissions
    pub roles: Vec<String>, // User roles
    pub scope: Option<String>, // OAuth scope
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub access_token_ttl: chrono::Duration,
    pub algorithm: Algorithm,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: "auth-platform".to_string(),
            audience: "auth-platform".to_string(),
            access_token_ttl: chrono::Duration::minutes(15), // 15 minutes as per requirements
            algorithm: Algorithm::RS256,
        }
    }
}

pub struct JwtService {
    config: JwtConfig,
    key_manager: KeyManager,
}

impl JwtService {
    pub fn new(config: JwtConfig, key_manager: KeyManager) -> Self {
        Self { config, key_manager }
    }

    /// Generate a new JWT access token with the given claims
    pub async fn generate_access_token(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        permissions: Vec<String>,
        roles: Vec<String>,
        scope: Option<String>,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + self.config.access_token_ttl;
        
        let claims = JwtClaims {
            sub: user_id.to_string(),
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            permissions,
            roles,
            scope,
        };

        let header = Header::new(self.config.algorithm);
        let encoding_key = self.key_manager.get_encoding_key().await
            .map_err(|e| JwtError::KeyError(e.to_string()))?;

        encode(&header, &claims, &encoding_key)
            .map_err(JwtError::EncodingError)
    }

    /// Validate and decode a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        let decoding_key = self.key_manager.get_decoding_key().await
            .map_err(|e| JwtError::KeyError(e.to_string()))?;

        let token_data = decode::<JwtClaims>(token, &decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::ValidationError { reason: e.to_string() },
            })?;

        Ok(token_data.claims)
    }

    /// Extract claims from token without validation (for introspection)
    pub fn extract_claims_unsafe(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let mut validation = Validation::new(self.config.algorithm);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        validation.insecure_disable_signature_validation();

        // Use a dummy key since we're not validating signature
        let dummy_key = DecodingKey::from_secret(b"dummy");
        
        let token_data = decode::<JwtClaims>(token, &dummy_key, &validation)
            .map_err(|_| JwtError::InvalidFormat)?;

        Ok(token_data.claims)
    }

    /// Check if token is expired
    pub fn is_token_expired(&self, claims: &JwtClaims) -> bool {
        let now = Utc::now().timestamp();
        claims.exp < now
    }

    /// Get token expiration time
    pub fn get_token_expiration(&self, claims: &JwtClaims) -> DateTime<Utc> {
        DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(Utc::now)
    }

    /// Get public keys as JWK Set
    pub fn get_jwk_set(&self) -> serde_json::Value {
        self.key_manager.get_jwk_set()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyManager;

    #[tokio::test]
    async fn test_jwt_generation_and_validation() {
        let config = JwtConfig::default();
        let key_manager = KeyManager::new_for_testing().await.unwrap();
        let jwt_service = JwtService::new(config, key_manager);

        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let permissions = vec!["read:users".to_string(), "write:users".to_string()];
        let roles = vec!["admin".to_string()];

        // Generate token
        let token = jwt_service
            .generate_access_token(user_id, tenant_id, permissions.clone(), roles.clone(), None)
            .await
            .unwrap();

        // Validate token
        let claims = jwt_service.validate_token(&token).await.unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.tenant_id, tenant_id.to_string());
        assert_eq!(claims.permissions, permissions);
        assert_eq!(claims.roles, roles);
        assert!(!jwt_service.is_token_expired(&claims));
    }

    #[tokio::test]
    async fn test_token_expiration() {
        let mut config = JwtConfig::default();
        config.access_token_ttl = chrono::Duration::milliseconds(1); // Very short TTL
        
        let key_manager = KeyManager::new_for_testing().await.unwrap();
        let jwt_service = JwtService::new(config, key_manager);

        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();

        let token = jwt_service
            .generate_access_token(user_id, tenant_id, vec![], vec![], None)
            .await
            .unwrap();

        // Wait for token to expire
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Token should be expired
        let result = jwt_service.validate_token(&token).await;
        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }
}