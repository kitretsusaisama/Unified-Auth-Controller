//! Multi-Channel Registration Handler
//!
//! Supports registration via:
//! - Email only
//! - Phone only
//! - Email + Phone (dual)

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use auth_core::error::AuthError;
use auth_core::models::user::{IdentifierType, PrimaryIdentifier};
use auth_core::models::validation::{normalize_phone, validate_email};
use auth_core::services::identity::IdentityService;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// Identifier type: "email", "phone", or "both"
    pub identifier_type: String,

    /// Email address (required if identifier_type is "email" or "both")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Phone number (required if identifier_type is "phone" or "both")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Primary identifier for login: "email" or "phone" (required if both provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_identifier: Option<String>,

    /// Password (optional for passwordless registration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Tenant ID
    pub tenant_id: Uuid,

    /// Profile data
    #[serde(default)]
    pub profile: serde_json::Value,

    /// Whether to require verification before allowing login
    #[serde(default = "default_require_verification")]
    pub require_verification: bool,
}

fn default_require_verification() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub identifier_type: String,
    pub verification_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_sent_to: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

// ============================================================================
// Handler
// ============================================================================

/// POST /auth/register
///
/// Multi-channel user registration
pub async fn register(
    State(identity_service): State<Arc<IdentityService>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    // 1. Validate identifier type
    let identifier_type = match payload.identifier_type.as_str() {
        "email" => IdentifierType::Email,
        "phone" => IdentifierType::Phone,
        "both" => IdentifierType::Both,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid identifier_type. Must be 'email', 'phone', or 'both'"
                        .to_string(),
                    code: "AUTH_038".to_string(),
                    field: Some("identifier_type".to_string()),
                }),
            ));
        }
    };

    // 2. Validate required fields based on identifier_type
    match identifier_type {
        IdentifierType::Email => {
            if payload.email.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Email is required when identifier_type is 'email'".to_string(),
                        code: "AUTH_004".to_string(),
                        field: Some("email".to_string()),
                    }),
                ));
            }
        }
        IdentifierType::Phone => {
            if payload.phone.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Phone is required when identifier_type is 'phone'".to_string(),
                        code: "AUTH_004".to_string(),
                        field: Some("phone".to_string()),
                    }),
                ));
            }
        }
        IdentifierType::Both => {
            if payload.email.is_none() || payload.phone.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Both email and phone are required when identifier_type is 'both'"
                            .to_string(),
                        code: "AUTH_004".to_string(),
                        field: None,
                    }),
                ));
            }

            if payload.primary_identifier.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "primary_identifier is required when identifier_type is 'both'"
                            .to_string(),
                        code: "AUTH_004".to_string(),
                        field: Some("primary_identifier".to_string()),
                    }),
                ));
            }
        }
    };

    // 3. Validate email format if provided
    if let Some(ref email) = payload.email {
        validate_email(email).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid email format".to_string(),
                    code: "AUTH_001".to_string(),
                    field: Some("email".to_string()),
                }),
            )
        })?;
    }

    // 4. Validate and normalize phone if provided
    let normalized_phone = if let Some(ref phone) = payload.phone {
        Some(normalize_phone(phone).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Invalid phone format. Use E.164 format (e.g., +14155552671)"
                        .to_string(),
                    code: "AUTH_002".to_string(),
                    field: Some("phone".to_string()),
                }),
            )
        })?)
    } else {
        None
    };

    // 5. Determine primary identifier
    let primary_identifier = match identifier_type {
        IdentifierType::Email => PrimaryIdentifier::Email,
        IdentifierType::Phone => PrimaryIdentifier::Phone,
        IdentifierType::Both => match payload.primary_identifier.as_deref() {
            Some("email") => PrimaryIdentifier::Email,
            Some("phone") => PrimaryIdentifier::Phone,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Invalid primary_identifier. Must be 'email' or 'phone'".to_string(),
                        code: "AUTH_004".to_string(),
                        field: Some("primary_identifier".to_string()),
                    }),
                ));
            }
        },
    };

    // 6. Validate password (if provided)
    if let Some(ref password) = payload.password {
        if password.len() < 8 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Password must be at least 8 characters".to_string(),
                    code: "AUTH_003".to_string(),
                    field: Some("password".to_string()),
                }),
            ));
        }
    }

    // 7. Create user via identity service
    let create_request = auth_core::models::user::CreateUserRequest {
        identifier_type,
        email: payload.email,
        phone: normalized_phone.clone(),
        primary_identifier: Some(primary_identifier.clone()),
        password: payload.password,
        profile_data: Some(payload.profile),
        require_verification: Some(payload.require_verification),
    };

    let user = identity_service
        .register(create_request, payload.tenant_id)
        .await
        .map_err(|e| {
            // Map AuthError to API Error
            match e {
                AuthError::Conflict { message } => (
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: message,
                        code: "AUTH_005".to_string(),
                        field: None,
                    }),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Internal server error".to_string(),
                        code: "AUTH_026".to_string(),
                        field: None,
                    }),
                ),
            }
        })?;

    // 8. Send verification if required
    let verification_sent_to = if payload.require_verification {
        match primary_identifier {
            // Use the original (cloned) value
            PrimaryIdentifier::Email => user.email.clone(),
            PrimaryIdentifier::Phone => user.phone.clone(),
        }
    } else {
        None
    };

    // 9. Return success response
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            status: user.status.to_string(), // Ensure UserStatus implements Display or ToString, or map manually
            email: user.email,
            phone: user.phone,
            identifier_type: payload.identifier_type,
            verification_required: payload.require_verification,
            verification_sent_to,
            created_at: user.created_at.to_rfc3339(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_only() {
        let req = RegisterRequest {
            identifier_type: "email".to_string(),
            email: Some("user@example.com".to_string()),
            phone: None,
            primary_identifier: None,
            password: Some("password123".to_string()),
            tenant_id: Uuid::new_v4(),
            profile: serde_json::json!({}),
            require_verification: true,
        };

        assert_eq!(req.identifier_type, "email");
        assert!(req.email.is_some());
    }

    #[test]
    fn test_validate_phone_only() {
        let req = RegisterRequest {
            identifier_type: "phone".to_string(),
            email: None,
            phone: Some("+14155552671".to_string()),
            primary_identifier: None,
            password: Some("password123".to_string()),
            tenant_id: Uuid::new_v4(),
            profile: serde_json::json!({}),
            require_verification: true,
        };

        assert_eq!(req.identifier_type, "phone");
        assert!(req.phone.is_some());
    }

    #[test]
    fn test_normalize_phone() {
        let phone = "+1 (415) 555-2671";
        let normalized = normalize_phone(phone).unwrap();
        assert_eq!(normalized, "+14155552671");
    }
}
