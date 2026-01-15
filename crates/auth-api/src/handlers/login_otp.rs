//! Login with OTP and Lazy Registration Support
//! 
//! Handles:
//! - Login via OTP (Passwordless)
//! - Auto-creation of account if lazy registration is enabled and user not found

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use auth_core::services::{
    otp_service::{OtpService, OtpPurpose},
    identity::IdentityService,
    lazy_registration::LazyRegistrationService,
};
use auth_db::repositories::otp_repository::OtpRepository;
use crate::error::ApiError;
use auth_core::error::{AuthError, TokenErrorKind};
use auth_core::models::user::IdentifierType;

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LoginOtpRequest {
    pub identifier: String,
    pub otp: String,
    pub session_id: Uuid,
    pub tenant_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub user_id: Uuid,
    pub is_new_user: bool,
    pub is_profile_complete: bool,
}

// ============================================================================
// Handler
// ============================================================================

/// POST /auth/login/otp
/// 
/// Login using OTP. If user doesn't exist, try lazy registration.
pub async fn login_with_otp(
    State(otp_service): State<Arc<OtpService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    State(lazy_service): State<Arc<LazyRegistrationService>>,
    State(identity_service): State<Arc<IdentityService>>, 
    Json(payload): Json<LoginOtpRequest>,
) -> Result<impl IntoResponse, ApiError> {
    
    // 1. Verify OTP
    // Fetch session
    let (session, otp_hash) = otp_repo
        .find_by_id(payload.session_id)
        .await?
        .ok_or(ApiError::new(AuthError::TokenError { kind: TokenErrorKind::Invalid }))?;

    // Validate Session
    if session.identifier != payload.identifier {
         return Err(ApiError::new(AuthError::ValidationError { message: "Identifier mismatch".to_string() }));
    }
    if otp_service.is_expired(&session) { // is_expired returns bool directly
        return Err(ApiError::new(AuthError::TokenError { kind: TokenErrorKind::Expired }));
    }
    // Verify hash
    if !otp_service.verify_otp(&payload.otp, &otp_hash).map_err(|_| ApiError::new(AuthError::InternalError))? {
        otp_repo.increment_attempts(payload.session_id).await.map_err(|_| ApiError::new(AuthError::InternalError))?;
        return Err(ApiError::new(AuthError::InvalidCredentials));
    }

    // Mark verified
    otp_repo.mark_verified(payload.session_id).await.map_err(|_| ApiError::new(AuthError::InternalError))?;

    // 2. Identify User
    let identifier_type = auth_core::models::validation::detect_identifier_type(&payload.identifier);
    
    // 3. Get or Create User (Lazy)
    let (user, created) = lazy_service.get_or_create_user(
        payload.tenant_id,
        &payload.identifier,
        identifier_type
    ).await.map_err(|_| ApiError::new(AuthError::InternalError))?;

    // 4. Issue Tokens
    let auth_response = identity_service.issue_tokens_for_user(&user, payload.tenant_id).await.map_err(|_| ApiError::new(AuthError::InternalError))?;
    
    // Check if profile is complete (assumes we added this check in User model or helper)
    // For now we check the profile_data json or the flag if available on User struct
    let is_profile_complete = user.profile_data.get("is_profile_complete").and_then(|v| v.as_bool()).unwrap_or(true);

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            user_id: user.id,
            is_new_user: created,
            is_profile_complete, 
        }),
    ))
}
