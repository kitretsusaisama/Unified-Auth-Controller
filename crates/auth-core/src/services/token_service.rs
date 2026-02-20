//! Token management service

use crate::error::{AuthError, TokenErrorKind};
use crate::models::{AccessToken, RefreshToken, TokenPair, Claims};
use auth_crypto::{JwtService, JwtConfig, JwtClaims, JwtError, KeyManager};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

const DEFAULT_CACHE_CAPACITY: usize = 10_000;

/// Trait for refresh token persistent storage
#[async_trait::async_trait]
pub trait RefreshTokenStore: Send + Sync {
    async fn create(&self, token: RefreshToken) -> Result<(), AuthError>;
    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthError>;
    async fn revoke(&self, token_id: Uuid) -> Result<(), AuthError>;
    async fn revoke_family(&self, family_id: Uuid) -> Result<(), AuthError>;
}

/// Trait for revoked access token storage (blacklist)
#[async_trait::async_trait]
pub trait RevokedTokenStore: Send + Sync {
    async fn add_to_blacklist(&self, jti: Uuid, user_id: Uuid, tenant_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), AuthError>;
    async fn is_revoked(&self, jti: Uuid) -> Result<bool, AuthError>;
}

#[async_trait::async_trait]
pub trait TokenProvider: Send + Sync {
    async fn issue_access_token(&self, claims: Claims) -> Result<AccessToken, AuthError>;
    async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> Result<RefreshToken, AuthError>;
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
    async fn revoke_token(&self, token_id: Uuid, user_id: Uuid, tenant_id: Uuid) -> Result<(), AuthError>;
    async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
    async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResponse, AuthError>;
} // End trait

// ... skip to InMemory impl ...
// Note: Can't skip in replace, I must do multiple chunks or one big valid block, but I can't leave gaps in a single replacement if using StartLine/EndLine unless I replace the whole range.
// I will use multi_replace for accuracy.
// But first let's see why previous failed. "TargetContent cannot be empty".
// I provided TargetContent.

// Let's use TokenEngine impl.

#[derive(Debug, Clone)]
pub struct TokenIntrospectionResponse {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub token_type: Option<String>,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    pub nbf: Option<i64>,
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub iss: Option<String>,
    pub jti: Option<String>,
}

pub struct TokenEngine {
    jwt_service: JwtService,
    revoked_token_store: Arc<dyn RevokedTokenStore>,
    refresh_token_store: Arc<dyn RefreshTokenStore>,
}

// In-memory implementations for testing/default
pub struct InMemoryRefreshTokenStore {
    tokens: Arc<RwLock<LruCache<String, RefreshToken>>>,
}

impl InMemoryRefreshTokenStore {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_CAPACITY)
                .expect("DEFAULT_CACHE_CAPACITY must be non-zero"));
        Self {
            tokens: Arc::new(RwLock::new(LruCache::new(cap))),
        }
    }
}

#[async_trait::async_trait]
impl RefreshTokenStore for InMemoryRefreshTokenStore {
    async fn create(&self, token: RefreshToken) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().await;
        tokens.put(token.token_hash.clone(), token);
        Ok(())
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthError> {
        let mut tokens = self.tokens.write().await;  // LRU needs mut for get
        Ok(tokens.get(hash).cloned())
    }

    async fn revoke(&self, token_id: Uuid) -> Result<(), AuthError> {
        // For LRU cache, scan all entries (inefficient but necessary for in-memory)
        let mut tokens = self.tokens.write().await;
        
        // Find the matching token
        let mut found_hash: Option<String> = None;
        for (hash, token) in tokens.iter() {
            if token.id == token_id {
                found_hash = Some(hash.clone());
                break;
            }
        }
        
        // Soft delete by setting revoked_at
        if let Some(hash) = found_hash {
            if let Some(token) = tokens.get_mut(&hash) {
                token.revoked_at = Some(Utc::now());
            }
        }
        Ok(())
    }

    async fn revoke_family(&self, _family_id: Uuid) -> Result<(), AuthError> {
        // TODO: Implement family revocation
        Ok(())
    }
}

pub struct InMemoryRevokedTokenStore {
    revoked: Arc<RwLock<LruCache<Uuid, DateTime<Utc>>>>,
}

impl InMemoryRevokedTokenStore {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_CAPACITY)
                .expect("DEFAULT_CACHE_CAPACITY must be non-zero"));
        Self {
            revoked: Arc::new(RwLock::new(LruCache::new(cap))),
        }
    }
}

#[async_trait::async_trait]
impl RevokedTokenStore for InMemoryRevokedTokenStore {
    async fn add_to_blacklist(&self, jti: Uuid, _user_id: Uuid, _tenant_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), AuthError> {
        let mut revoked = self.revoked.write().await;
        revoked.put(jti, expires_at);
        Ok(())
    }

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, AuthError> {
        let mut revoked = self.revoked.write().await;  // LRU needs mut for get
        if let Some(expiry) = revoked.get(&jti) {
            Ok(*expiry > Utc::now())
        } else {
            Ok(false)
        }
    }
}

impl TokenEngine {
    pub async fn new() -> Result<Self, AuthError> {
        let config = JwtConfig::default();
        let key_manager = KeyManager::new().await
            .map_err(|e| AuthError::ConfigurationError { 
                message: format!("Failed to initialize key manager: {}", e) 
            })?;
        
        Ok(Self {
            jwt_service: JwtService::new(config, key_manager),
            revoked_token_store: Arc::new(InMemoryRevokedTokenStore::new(10_000)),
            refresh_token_store: Arc::new(InMemoryRefreshTokenStore::new(10_000)),
        })
    }

    pub async fn new_with_stores(
        revoked_store: Arc<dyn RevokedTokenStore>,
        refresh_store: Arc<dyn RefreshTokenStore>,
    ) -> Result<Self, AuthError> {
        let config = JwtConfig::default();
        let key_manager = KeyManager::new().await
            .map_err(|e| AuthError::ConfigurationError { 
                message: format!("Failed to initialize key manager: {}", e) 
            })?;
        
        Ok(Self {
            jwt_service: JwtService::new(config, key_manager),
            revoked_token_store: revoked_store,
            refresh_token_store: refresh_store,
        })
    }

    /// Clean up expired tokens (No-op in trait-based implementation as DB handles it)
    async fn cleanup_expired_tokens(&self) {}
}

#[async_trait::async_trait]
impl TokenProvider for TokenEngine {
    async fn issue_access_token(&self, claims: Claims) -> Result<AccessToken, AuthError> {
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::TokenError { kind: TokenErrorKind::Invalid })?;
        
        let tenant_id = Uuid::parse_str(&claims.tenant_id)
            .map_err(|_| AuthError::TokenError { kind: TokenErrorKind::Invalid })?;

        let token = self.jwt_service
            .generate_access_token(
                user_id,
                tenant_id,
                claims.permissions,
                claims.roles,
                None, // scope - OAuth specific, optional
            )
            .await
            .map_err(|e| match e {
                JwtError::EncodingError(_) => AuthError::TokenError { 
                    kind: TokenErrorKind::Invalid 
                },
                JwtError::KeyError(msg) => AuthError::ConfigurationError { message: msg },
                _ => AuthError::TokenError { kind: TokenErrorKind::Invalid },
            })?;

        Ok(AccessToken {
            token,
            token_type: "Bearer".to_string(),
            expires_in: 900,
            scope: None,
        })
    }

    async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> Result<RefreshToken, AuthError> {
        let token_id = Uuid::new_v4();
        let token_family = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::days(30);
        
        // Generate a secure random token
        let token_hash = format!("rt_{}", Uuid::new_v4());
        
        let refresh_token = RefreshToken {
            id: token_id,
            user_id,
            tenant_id,
            token_family,
            token_hash: token_hash.clone(),
            device_fingerprint: None,
            user_agent: None,
            ip_address: None,
            expires_at,
            revoked_at: None,
            revoked_reason: None,
            created_at: now,
        };

        self.refresh_token_store.create(refresh_token.clone()).await?;

        Ok(refresh_token)
    }

    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let jwt_claims = self.jwt_service
            .validate_token(token)
            .await
            .map_err(|e| match e {
                JwtError::TokenExpired => AuthError::TokenError { 
                    kind: TokenErrorKind::Expired 
                },
                JwtError::ValidationError { .. } => AuthError::TokenError { 
                    kind: TokenErrorKind::Invalid 
                },
                _ => AuthError::TokenError { kind: TokenErrorKind::Invalid },
            })?;

        // Check blacklist using JTI
        if let Ok(jti) = Uuid::parse_str(&jwt_claims.jti) {
             if self.revoked_token_store.is_revoked(jti).await? {
                 return Err(AuthError::TokenError { kind: TokenErrorKind::Revoked });
             }
        }

        Ok(Claims {
            sub: jwt_claims.sub,
            iss: jwt_claims.iss,
            aud: jwt_claims.aud,
            exp: jwt_claims.exp,
            iat: jwt_claims.iat,
            nbf: jwt_claims.nbf,
            jti: jwt_claims.jti,
            tenant_id: jwt_claims.tenant_id,
            permissions: jwt_claims.permissions,
            roles: jwt_claims.roles,
        })
    }

    async fn revoke_token(&self, token_id: Uuid, user_id: Uuid, tenant_id: Uuid) -> Result<(), AuthError> {
        let expiry = Utc::now() + Duration::hours(24);
        self.revoked_token_store.add_to_blacklist(token_id, user_id, tenant_id, expiry).await?;
        // Also revoke refresh token if it exists
        let _ = self.refresh_token_store.revoke(token_id).await;
        Ok(())
    }

    async fn refresh_tokens(&self, refresh_token_hash: &str) -> Result<TokenPair, AuthError> {
        let token_data = self.refresh_token_store.find_by_hash(refresh_token_hash).await?
            .ok_or(AuthError::TokenError { kind: TokenErrorKind::Invalid })?;

        if token_data.expires_at < Utc::now() {
            return Err(AuthError::TokenError { 
                kind: TokenErrorKind::Expired 
            });
        }
        
        if token_data.revoked_at.is_some() {
             return Err(AuthError::TokenError { kind: TokenErrorKind::Revoked });
        }

        // Rotate
        self.refresh_token_store.revoke(token_data.id).await?;

        let claims = Claims {
            sub: token_data.user_id.to_string(),
            iss: "auth-platform".to_string(),
            aud: "auth-platform".to_string(),
            exp: (Utc::now() + Duration::minutes(15)).timestamp(),
            iat: Utc::now().timestamp(),
            nbf: Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: token_data.tenant_id.to_string(),
            permissions: vec![],
            roles: vec![],
        };

        let access_token = self.issue_access_token(claims).await?;
        let new_refresh_token = self.issue_refresh_token(token_data.user_id, token_data.tenant_id).await?;

        Ok(TokenPair {
            access_token,
            refresh_token: new_refresh_token.token_hash,
        })
    }

    async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResponse, AuthError> {
        let claims_result = self.jwt_service.extract_claims_unsafe(token);
        
        let claims = match claims_result {
             Ok(c) => c,
             Err(_) => return Ok(TokenIntrospectionResponse {
                 active: false, scope: None, client_id: None, username: None, 
                 token_type: None, exp: None, iat: None, nbf: None, sub: None, 
                 aud: None, iss: None, jti: None
             })
        };

        let mut is_revoked = false;
        if let Ok(jti) = Uuid::parse_str(&claims.jti) {
             if self.revoked_token_store.is_revoked(jti).await.unwrap_or(false) {
                 is_revoked = true;
             }
        }
        
        let is_expired = self.jwt_service.is_token_expired(&claims);
        let active = !is_revoked && !is_expired;

        Ok(TokenIntrospectionResponse {
            active,
            scope: None,
            client_id: None,
            username: Some(claims.sub.clone()),
            token_type: Some("Bearer".to_string()),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            nbf: Some(claims.nbf),
            sub: Some(claims.sub),
            aud: Some(claims.aud),
            iss: Some(claims.iss),
            jti: Some(claims.jti),
        })
    }
}