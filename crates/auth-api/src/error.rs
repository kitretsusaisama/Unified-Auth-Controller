use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use auth_core::error::AuthError;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

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
        let (status, code, message) = match &self.inner {
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "The provided credentials are invalid".to_string(),
            ),
            AuthError::Unauthorized { message } => (
                StatusCode::FORBIDDEN,
                "UNAUTHORIZED",
                message.clone(),
            ),
            AuthError::Conflict { message } => (
                StatusCode::CONFLICT,
                "CONFLICT",
                message.clone(),
            ),
            AuthError::ValidationError { message } => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                message.clone(),
            ),
            AuthError::UserNotFound { .. } => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "User not found".to_string(),
            ),
            AuthError::AccountLocked { reason } => (
                StatusCode::LOCKED,
                "ACCOUNT_LOCKED",
                reason.clone(),
            ),
            AuthError::RateLimitExceeded { limit, window } => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                format!("Rate limit exceeded: {} requests per {}", limit, window),
            ),
            AuthError::TokenError { kind } => (
                StatusCode::UNAUTHORIZED,
                "TOKEN_ERROR",
                format!("Token error: {:?}", kind),
            ),
            AuthError::PasswordPolicyViolation { errors } => (
                StatusCode::BAD_REQUEST,
                "PASSWORD_POLICY_VIOLATION",
                errors.join(", "),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };

        let error_response = ErrorResponse {
            code: code.to_string(),
            message,
            fields: None,
            request_id: self.request_id.map(|id| id.to_string()),
        };

        (status, Json(error_response)).into_response()
    }
}

impl From<AuthError> for ApiError {
    fn from(inner: AuthError) -> Self {
        ApiError::new(inner)
    }
}