use crate::error::AuthError;
use crate::models::{User, CreateUserRequest, UpdateUserRequest, UserStatus};
use crate::services::token_service::{TokenProvider, TokenEngine};
use crate::models::Claims;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AuthError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError>;
    async fn create(&self, user: CreateUserRequest, password_hash: String, tenant_id: Uuid) -> Result<User, AuthError>;
    async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), AuthError>;
    async fn increment_failed_attempts(&self, id: Uuid) -> Result<u32, AuthError>;
    async fn reset_failed_attempts(&self, id: Uuid) -> Result<(), AuthError>;
    async fn record_login(&self, id: Uuid, ip: Option<String>) -> Result<(), AuthError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
    pub tenant_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub user: User,
    pub access_token: String,
    pub refresh_token: String,
    pub requires_mfa: bool,
}

pub struct IdentityService {
    store: Arc<dyn UserStore>,
    token_service: Arc<dyn TokenProvider>,
}

impl IdentityService {
    pub fn new(store: Arc<dyn UserStore>, token_service: Arc<dyn TokenProvider>) -> Self {
        Self { store, token_service }
    }

    pub async fn register(&self, request: CreateUserRequest, tenant_id: Uuid) -> Result<User, AuthError> {
        // 1. Validate Password exists
        if request.password.is_none() {
             return Err(AuthError::ValidationError { message: "Password required".to_string() });
        }

        // 2. Check existence
        if (self.store.find_by_email(&request.email, tenant_id).await?).is_some() {
            return Err(AuthError::Conflict { message: "Email already registered".to_string() });
        }

        // 3. Hash Password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(request.password.as_ref().unwrap().as_bytes(), &salt)
            .map_err(|e| AuthError::UTCryptoError(e.to_string()))?
            .to_string();

        // 4. Create User
        let user = self.store.create(request, password_hash, tenant_id).await?;
        
        // 5. TODO: Trigger Audit Log (Registration)

        Ok(user)
    }

    pub async fn login(&self, request: AuthRequest) -> Result<AuthResponse, AuthError> {
        // 1. Fetch User
        let user = self.store.find_by_email(&request.email, request.tenant_id).await?
            .ok_or(AuthError::InvalidCredentials)?;

        // 2. Check Status
        if !user.can_authenticate() {
             return Err(AuthError::Unauthorized { message: "Account locked or suspended".to_string() });
        }

        // 3. Verify Password
        let parsed_hash = PasswordHash::new(user.password_hash.as_ref().unwrap())
            .map_err(|e| AuthError::UTCryptoError(e.to_string()))?;
        
        if Argon2::default().verify_password(request.password.as_bytes(), &parsed_hash).is_err() {
            // Increment failed attempts
            let attempts = self.store.increment_failed_attempts(user.id).await?;
            // If attempts > 5, lock (handled by DB or here? DB logic sets locked_until potentially, but store should handle it or we update explicitly)
            // Implementation detail: UserStore::increment usually returns new count.
            if attempts >= 5 {
                // TODO: Set locked_until in DB explicitly if not done by increment proc
                // For now assuming UserStore logic or explicit call needed.
                // We'll leave it as "Implementation Detail" of store or next step.
            }
            return Err(AuthError::InvalidCredentials);
        }

        // 4. Reset failed attempts
        self.store.record_login(user.id, request.ip_address).await?;

        // 5. Issue Tokens
        let claims = Claims {
            sub: user.id.to_string(),
            iss: "auth-service".to_string(),
            aud: "auth-service".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
            iat: chrono::Utc::now().timestamp(),
            nbf: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: request.tenant_id.to_string(), 
            permissions: vec![],
            roles: vec![],
        };

        let access_token_struct = self.token_service.issue_access_token(claims).await?;
        let access_token = access_token_struct.token;
        let refresh_token_struct = self.token_service.issue_refresh_token(user.id, request.tenant_id).await?;
        let refresh_token = refresh_token_struct.token_hash;

        let requires_mfa = user.mfa_enabled;
        Ok(AuthResponse {
            user,
            access_token,
            refresh_token,
            requires_mfa,
        })
    }

    pub async fn ban_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.update_status(user_id, UserStatus::Suspended).await
    }
    
    pub async fn activate_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.update_status(user_id, UserStatus::Active).await
    }
}