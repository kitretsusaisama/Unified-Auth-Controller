use crate::error::AuthError;
use crate::models::user::{IdentifierType, PrimaryIdentifier};
use crate::models::Claims;
use crate::models::{CreateUserRequest, UpdateUserRequest, User, UserStatus};
use crate::services::token_service::TokenProvider;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AuthError>;
    async fn find_by_phone(&self, phone: &str, tenant_id: Uuid) -> Result<Option<User>, AuthError>;
    async fn find_by_identifier(
        &self,
        identifier: &str,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError>;
    async fn create(
        &self,
        user: CreateUserRequest,
        password_hash: String,
        tenant_id: Uuid,
    ) -> Result<User, AuthError>;
    async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), AuthError>;
    async fn increment_failed_attempts(&self, id: Uuid) -> Result<u32, AuthError>;
    async fn reset_failed_attempts(&self, id: Uuid) -> Result<(), AuthError>;
    async fn record_login(&self, id: Uuid, ip: Option<String>) -> Result<(), AuthError>;
    async fn update(&self, user: UpdateUserRequest) -> Result<User, AuthError>;
    async fn update_password_hash(&self, id: Uuid, password_hash: String) -> Result<(), AuthError>;
    async fn set_email_verified(&self, id: Uuid, verified: bool) -> Result<(), AuthError>;
    async fn set_phone_verified(&self, id: Uuid, verified: bool) -> Result<(), AuthError>;
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
        Self {
            store,
            token_service,
        }
    }

    pub async fn register(
        &self,
        request: CreateUserRequest,
        tenant_id: Uuid,
    ) -> Result<User, AuthError> {
        // 1. Validate Password exists logic (optional based on use case, but for manual register it's usually required)
        if request.password.is_none() {
            return Err(AuthError::ValidationError {
                message: "Password required".to_string(),
            });
        }

        // 2. Check existence
        if let Some(ref email) = request.email {
            if (self.store.find_by_email(email, tenant_id).await?).is_some() {
                return Err(AuthError::Conflict {
                    message: "Email already registered".to_string(),
                });
            }
        }

        if let Some(ref phone) = request.phone {
            if (self.store.find_by_phone(phone, tenant_id).await?).is_some() {
                return Err(AuthError::Conflict {
                    message: "Phone already registered".to_string(),
                });
            }
        }

        // 3. Hash Password (offload to blocking thread to prevent executor starvation)
        let password_clone = request.password.as_ref().unwrap().clone();
        let password_hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            argon2
                .hash_password(password_clone.as_bytes(), &salt)
                .map(|h| h.to_string())
        })
        .await
        .map_err(|_| AuthError::InternalError)?
        .map_err(|e| AuthError::UTCryptoError(e.to_string()))?;

        // 4. Create User
        let user = self.store.create(request, password_hash, tenant_id).await?;

        // 5. TODO: Trigger Audit Log (Registration)

        Ok(user)
    }

    pub async fn login(&self, request: AuthRequest) -> Result<AuthResponse, AuthError> {
        // 1. Fetch User
        let user = self
            .store
            .find_by_email(&request.email, request.tenant_id)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // 2. Check Status
        if !user.can_authenticate() {
            return Err(AuthError::Unauthorized {
                message: "Account locked or suspended".to_string(),
            });
        }

        // 3. Verify Password (offload to blocking thread to prevent executor starvation)
        let password_clone = request.password.clone();
        let hash_clone = user.password_hash.as_ref().unwrap().clone();

        let is_valid = tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash_clone).ok()?;
            Some(
                Argon2::default()
                    .verify_password(password_clone.as_bytes(), &parsed_hash)
                    .is_ok(),
            )
        })
        .await
        .map_err(|_| AuthError::InternalError)?
        .unwrap_or(false);

        if !is_valid {
            // Increment failed attempts
            let attempts = self.store.increment_failed_attempts(user.id).await?;
            if attempts >= 5 {
                // TODO: Verify UserStore::increment_failed_attempts sets locked_until
            }
            return Err(AuthError::InvalidCredentials);
        }

        // 4. Reset failed attempts
        self.store.record_login(user.id, request.ip_address).await?;

        // 5. Issue Tokens
        self.issue_tokens_for_user(&user, request.tenant_id).await
    }

    /// Issue access and refresh tokens for a newly authenticated user
    pub async fn issue_tokens_for_user(
        &self,
        user: &User,
        tenant_id: Uuid,
    ) -> Result<AuthResponse, AuthError> {
        let claims = Claims {
            sub: user.id.to_string(),
            iss: "auth-service".to_string(),
            aud: "auth-service".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
            iat: chrono::Utc::now().timestamp(),
            nbf: chrono::Utc::now().timestamp(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            permissions: vec![],
            roles: vec![],
        };

        let access_token_struct = self.token_service.issue_access_token(claims).await?;
        let access_token = access_token_struct.token;
        let refresh_token_struct = self
            .token_service
            .issue_refresh_token(user.id, tenant_id)
            .await?;
        let refresh_token = refresh_token_struct.token_hash;

        let requires_mfa = user.mfa_enabled;
        Ok(AuthResponse {
            user: user.clone(),
            access_token,
            refresh_token,
            requires_mfa,
        })
    }

    pub async fn ban_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store
            .update_status(user_id, UserStatus::Suspended)
            .await
    }

    pub async fn activate_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.update_status(user_id, UserStatus::Active).await
    }

    /// Find a user by any supported identifier (email or phone)
    pub async fn find_user_by_identifier(
        &self,
        tenant_id: Uuid,
        identifier: &str,
    ) -> Result<Option<User>, AuthError> {
        self.store.find_by_identifier(identifier, tenant_id).await
    }

    /// Create a new user lazily with minimal information
    pub async fn create_lazy_user(
        &self,
        tenant_id: Uuid,
        identifier: &str,
        identifier_type: IdentifierType,
    ) -> Result<User, AuthError> {
        // Construct basic user request
        let (email, phone, primary) = match identifier_type {
            IdentifierType::Email => (Some(identifier.to_string()), None, PrimaryIdentifier::Email),
            IdentifierType::Phone => (None, Some(identifier.to_string()), PrimaryIdentifier::Phone),
            _ => {
                return Err(AuthError::ValidationError {
                    message: "Invalid identifier type for lazy registration".to_string(),
                })
            } // 'Both' not supported for lazy yet
        };

        // For lazy users, we might set an unusable password or handled at DB level
        // Here we generate a random 32-char string to ensure no one can guess it
        let temp_password = Uuid::new_v4().to_string();

        let password_hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(temp_password.as_bytes(), &salt)
                .map(|h| h.to_string())
        })
        .await
        .map_err(|_| AuthError::InternalError)?
        .map_err(|e| AuthError::UTCryptoError(e.to_string()))?;

        let request = CreateUserRequest {
            identifier_type: identifier_type.clone(),
            email,
            phone,
            primary_identifier: Some(primary),
            password: None, // We manually hashed it above, so we pass None here (Wait, create expects CreateUserRequest, but also a password_hash string. The CreateUserRequest's password field is mostly for pre-hash validation if needed, but here we don't need it)
            // Actually, CreateUserRequest usually carries the password for validation.
            // In register(), we use request.password.
            // In create_lazy_user, we can set request.password to None, and pass the hash to store.create.
            profile_data: Some(json!({
                "source": "lazy_registration",
                "is_profile_complete": false
            })),
            require_verification: Some(true),
        };

        self.store.create(request, password_hash, tenant_id).await
    }

    /// Update user password
    pub async fn update_password(
        &self,
        user_id: Uuid,
        new_password: String,
    ) -> Result<(), AuthError> {
        // Hash new password
        let password_hash = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(new_password.as_bytes(), &salt)
                .map(|h| h.to_string())
        })
        .await
        .map_err(|_| AuthError::InternalError)?
        .map_err(|e| AuthError::UTCryptoError(e.to_string()))?;

        self.store
            .update_password_hash(user_id, password_hash)
            .await
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AuthError> {
        self.store
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        profile_data: serde_json::Value,
    ) -> Result<User, AuthError> {
        let update_request = UpdateUserRequest {
            id: user_id,
            email: None,
            phone: None,
            profile_data: Some(profile_data),
            preferences: None,
        };
        self.store.update(update_request).await
    }

    /// Mark email as verified
    pub async fn mark_email_verified(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.set_email_verified(user_id, true).await
    }

    /// Mark phone as verified
    pub async fn mark_phone_verified(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.set_phone_verified(user_id, true).await
    }
}
