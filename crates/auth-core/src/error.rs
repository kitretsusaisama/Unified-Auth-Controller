//! Error types for the authentication system
//!
//! Implements MNC-grade error handling with standard error codes.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Authorization denied: {permission} on {resource}")]
    AuthorizationDenied {
        permission: String,
        resource: String,
    },

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

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Internal error")]
    InternalError,

    #[error("Credential error: {message}")]
    CredentialError { message: String },

    #[error("Password policy violation: {errors:?}")]
    PasswordPolicyViolation { errors: Vec<String> },

    #[error("Account locked: {reason}")]
    AccountLocked { reason: String },

    #[error("Account suspended")]
    AccountSuspended,

    #[error("Account deleted")]
    AccountDeleted,

    #[error("Password expired")]
    PasswordExpired,

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },

    #[error("Crypto error: {0}")]
    UTCryptoError(String),

    #[error("Session not found")]
    SessionNotFound,

    #[error("Invalid OTP")]
    InvalidOtp,

    #[error("OTP expired")]
    OtpExpired,

    #[error("Circuit breaker open: {service}")]
    CircuitBreakerOpen { service: String },
}

#[derive(Debug, Clone)]
pub enum TokenErrorKind {
    Expired,
    Invalid,
    Revoked,
    MalformedSignature,
    UnsupportedAlgorithm,
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            AuthError::ValidationError { .. } => "AUTH_001", // And 002, 003, 004 depending on context, using generic 001 for now
            AuthError::Conflict { .. } => "AUTH_005",
            AuthError::InvalidCredentials => "AUTH_007",
            AuthError::InvalidOtp => "AUTH_008",
            AuthError::OtpExpired => "AUTH_009",
            AuthError::AccountLocked { .. } => "AUTH_011",
            AuthError::AccountSuspended => "AUTH_012",
            AuthError::AccountDeleted => "AUTH_013",
            AuthError::RateLimitExceeded { .. } => "AUTH_017", // Or 018, 040
            AuthError::TokenError { kind } => match kind {
                TokenErrorKind::Expired => "AUTH_021",
                TokenErrorKind::Revoked => "AUTH_022",
                _ => "AUTH_020",
            },
            AuthError::AuthorizationDenied { .. } => "AUTH_023",
            AuthError::Unauthorized { .. } => "AUTH_023",
            AuthError::UserNotFound => "AUTH_024",
            AuthError::SessionNotFound => "AUTH_025",
            AuthError::DatabaseError { .. } => "AUTH_026",
            AuthError::ExternalServiceError { .. } => "AUTH_027", // Or 028
            AuthError::CircuitBreakerOpen { .. } => "AUTH_046",
            AuthError::InternalError => "AUTH_026", // Generic internal
            AuthError::UTCryptoError(_) => "AUTH_044",
            AuthError::ConfigurationError { .. } => "AUTH_026",
            _ => "AUTH_026", // Default to internal DB type error
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AuthError::UserNotFound,
            _ => AuthError::DatabaseError {
                message: err.to_string(),
            },
        }
    }
}

impl From<auth_crypto::JwtError> for AuthError {
    fn from(err: auth_crypto::JwtError) -> Self {
        match err {
            auth_crypto::JwtError::TokenExpired => AuthError::TokenError {
                kind: TokenErrorKind::Expired,
            },
            auth_crypto::JwtError::ValidationError { .. } => AuthError::TokenError {
                kind: TokenErrorKind::Invalid,
            },
            auth_crypto::JwtError::EncodingError(_) => AuthError::TokenError {
                kind: TokenErrorKind::Invalid,
            },
            auth_crypto::JwtError::KeyError(msg) => AuthError::ConfigurationError { message: msg },
            _ => AuthError::TokenError {
                kind: TokenErrorKind::Invalid,
            },
        }
    }
}
impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        AuthError::ValidationError {
            message: err.to_string(),
        }
    }
}
impl From<crate::services::otp_delivery::DeliveryError> for AuthError {
    fn from(err: crate::services::otp_delivery::DeliveryError) -> Self {
        use crate::services::otp_delivery::DeliveryError;
        match err {
            DeliveryError::CircuitBreakerOpen(s) => AuthError::CircuitBreakerOpen { service: s },
            _ => AuthError::ExternalServiceError {
                service: "otp_delivery".to_string(),
                error: err.to_string(),
            },
        }
    }
}
impl From<crate::services::otp_service::OtpError> for AuthError {
    fn from(err: crate::services::otp_service::OtpError) -> Self {
        use crate::services::otp_service::OtpError;
        match err {
            OtpError::Invalid | OtpError::GenerationFailed => AuthError::InvalidOtp, // Generation failure internal really
            OtpError::Expired => AuthError::OtpExpired,
            OtpError::NotFound => AuthError::InvalidOtp,
            OtpError::MaxAttemptsExceeded => AuthError::RateLimitExceeded {
                limit: 5,
                window: "session".to_string(),
            }, // Mapping rough
            _ => AuthError::InternalError,
        }
    }
}
