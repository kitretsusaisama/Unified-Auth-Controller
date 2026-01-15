//! Error types for the authentication system

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    
    #[error("Authorization denied: {permission} on {resource}")]
    AuthorizationDenied { permission: String, resource: String },
    
    #[error("Token error: {kind:?}")]
    TokenError { kind: TokenErrorKind },
    
    #[error("Rate limit exceeded: {limit} requests per {window}")]
    RateLimitExceeded { limit: u32, window: String },
    
    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("External service error: {service} - {error}")]
    ExternalServiceError { service: String, error: String },
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Credential error: {message}")]
    CredentialError { message: String },
    
    #[error("Password policy violation: {errors:?}")]
    PasswordPolicyViolation { errors: Vec<String> },
    
    #[error("Account locked: {reason}")]
    AccountLocked { reason: String },
    
    #[error("Password expired")]
    PasswordExpired,
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Crypto error: {0}")]
    UTCryptoError(String),
}

#[derive(Debug, Clone)]
pub enum TokenErrorKind {
    Expired,
    Invalid,
    Revoked,
    MalformedSignature,
    UnsupportedAlgorithm,
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        AuthError::ValidationError {
            message: err.to_string(),
        }
    }
}