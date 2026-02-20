use crate::error::problem_details::ProblemDetails;
pub use auth_core::error::AuthError;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod problem_details;

/// Structured error response for API
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional field-level validation errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FieldError>>,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Field-level validation error
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FieldError {
    /// Field name
    pub field: String,
    /// Error message for this field
    pub message: String,
}

pub struct ApiError {
    pub inner: AuthError,
    pub request_id: Option<Uuid>,
}

impl ApiError {
    pub fn new(error: AuthError) -> Self {
        Self {
            inner: error,
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: Uuid) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let code = self.inner.code();

        let (status, message) = match &self.inner {
            AuthError::AuthenticationFailed { reason } => {
                (StatusCode::UNAUTHORIZED, reason.clone())
            }
            AuthError::AuthorizationDenied { permission, .. } => (
                StatusCode::FORBIDDEN,
                format!("Permission denied: {}", permission),
            ),
            AuthError::TokenError { .. } => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token".to_string(),
            ),
            AuthError::RateLimitExceeded { limit, window } => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("Rate limit exceeded: {} per {}", limit, window),
            ),
            AuthError::TenantNotFound { .. } => {
                (StatusCode::NOT_FOUND, "Tenant not found".to_string())
            }
            AuthError::ConfigurationError { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
            ),
            AuthError::ExternalServiceError { .. } => (
                StatusCode::BAD_GATEWAY,
                "External service error".to_string(),
            ),
            AuthError::DatabaseError { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AuthError::ValidationError { message } => (StatusCode::BAD_REQUEST, message.clone()),
            AuthError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AuthError::CredentialError { message } => (StatusCode::UNAUTHORIZED, message.clone()),
            AuthError::PasswordPolicyViolation { errors } => {
                (StatusCode::BAD_REQUEST, errors.join(", "))
            }
            AuthError::AccountLocked { reason } => (StatusCode::LOCKED, reason.clone()),
            AuthError::AccountSuspended => (StatusCode::FORBIDDEN, "Account suspended".to_string()),
            AuthError::AccountDeleted => (StatusCode::FORBIDDEN, "Account deleted".to_string()),
            AuthError::PasswordExpired => (StatusCode::FORBIDDEN, "Password expired".to_string()),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AuthError::Conflict { message } => (StatusCode::CONFLICT, message.clone()),
            AuthError::Unauthorized { message } => (StatusCode::UNAUTHORIZED, message.clone()),
            AuthError::UTCryptoError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cryptography error".to_string(),
            ),
            AuthError::SessionNotFound => {
                (StatusCode::UNAUTHORIZED, "Session not found".to_string())
            }
            AuthError::InvalidOtp => (StatusCode::BAD_REQUEST, "Invalid OTP".to_string()),
            AuthError::OtpExpired => (StatusCode::BAD_REQUEST, "OTP expired".to_string()),
            AuthError::CircuitBreakerOpen { service } => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Service unavailable: {}", service),
            ),
        };

        // Convert to RFC 7807 Problem Details
        let mut problem = ProblemDetails::new(status, message)
            .with_type(format!("https://auth.example.com/errors/{}", code))
            .with_extension("code", code);

        if let Some(req_id) = self.request_id {
            problem = problem.with_extension("request_id", req_id.to_string());
        }

        problem.into_response()
    }
}

impl From<AuthError> for ApiError {
    fn from(inner: AuthError) -> Self {
        ApiError::new(inner)
    }
}

impl From<auth_core::services::otp_service::OtpError> for ApiError {
    fn from(error: auth_core::services::otp_service::OtpError) -> Self {
        match error {
            auth_core::services::otp_service::OtpError::Invalid => {
                ApiError::new(auth_core::error::AuthError::InvalidOtp)
            }
            auth_core::services::otp_service::OtpError::Expired => {
                ApiError::new(auth_core::error::AuthError::OtpExpired)
            }
            auth_core::services::otp_service::OtpError::NotFound => {
                ApiError::new(auth_core::error::AuthError::InvalidOtp)
            }
            auth_core::services::otp_service::OtpError::MaxAttemptsExceeded => {
                ApiError::new(auth_core::error::AuthError::RateLimitExceeded {
                    limit: 5,
                    window: "session".to_string(),
                })
            }
            _ => ApiError::new(auth_core::error::AuthError::InternalError),
        }
    }
}

impl From<auth_core::services::otp_delivery::DeliveryError> for ApiError {
    fn from(error: auth_core::services::otp_delivery::DeliveryError) -> Self {
        use auth_core::services::otp_delivery::DeliveryError;
        match error {
            DeliveryError::CircuitBreakerOpen(s) => {
                ApiError::new(auth_core::error::AuthError::CircuitBreakerOpen { service: s })
            }
            _ => ApiError::new(auth_core::error::AuthError::ExternalServiceError {
                service: "otp_delivery".to_string(),
                error: error.to_string(),
            }),
        }
    }
}
