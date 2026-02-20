//! Verification Handlers
//! 
//! Endpoints for:
//! - Email Verification (Magic Link & Code)
//! - Phone Verification (SMS Code)

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use auth_core::services::{
    otp_service::{OtpService, OtpPurpose, DeliveryMethod, TokenType},
    otp_delivery::OtpDeliveryService,
    identity::IdentityService,
    rate_limiter::{RateLimiter, identifier_key},
};
use auth_db::repositories::otp_repository::OtpRepository;
use crate::error::ApiError;
use auth_core::error::{AuthError, TokenErrorKind};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SendVerificationRequest {
    pub user_id: Uuid,
    /// Optional: if verification is triggered for a specific identifier
    /// Defaults to user's primary/stored identifier if not provided
    pub start_process: bool, 
}

#[derive(Debug, Deserialize)]
pub struct SendEmailVerificationRequest {
    pub user_id: Uuid,
    pub email: Option<String>, // If not provided, use user's email
}

#[derive(Debug, Deserialize)]
pub struct SendPhoneVerificationRequest {
    pub user_id: Uuid,
    pub phone: Option<String>, // If not provided, use user's phone
}

#[derive(Debug, Serialize)]
pub struct VerificationResponse {
    pub verification_id: Uuid, // Session ID
    pub sent_to: String,
    pub method: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmVerificationRequest {
    pub user_id: Uuid,
    pub verification_id: Uuid,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct MagicLinkQuery {
    pub token: String,
    pub verification_id: Uuid,
}

// ============================================================================
// Email Verification
// ============================================================================

/// POST /auth/verify/email/send
/// Sends a Magic Link (and optional code) to the user's email
pub async fn send_email_verification(
    State(identity_service): State<Arc<IdentityService>>,
    State(otp_service): State<Arc<OtpService>>,
    State(otp_delivery): State<Arc<OtpDeliveryService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<SendEmailVerificationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    
    // 1. Fetch User
    let user = identity_service.get_user(payload.user_id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
    let email = payload.email.unwrap_or_else(|| user.email.clone().unwrap_or_default()); // Handle Option<String> properly
    
    if email.is_empty() {
         return Err(ApiError::new(auth_core::error::AuthError::ValidationError { message: "User has no email to verify".to_string() }));
    }
    
    if user.email_verified && user.email.as_ref().map_or(false, |e| e == &email) { // Check if email matches the one being verified
        return Err(ApiError::new(auth_core::error::AuthError::Conflict { message: "Email already verified".to_string() }));
    }

    // 2. Rate Limiting
    let limit_key = format!("verify_email:{}", user.id);
    let is_allowed: bool = rate_limiter.check_limit(&limit_key, "email_verification").await.map_err(|e| ApiError::new(auth_core::error::AuthError::InternalError))?;
    if !is_allowed {
         return Err(ApiError::new(auth_core::error::AuthError::RateLimitExceeded { limit: 3, window: "1 hour".to_string() }));
    }

    // 3. Generate Magic Link Token (High Entropy)
    // We use a longer token (32 chars) for Magic Links
    let token: String = otp_service.generate_token(TokenType::Alphanumeric, 32); // Removed .await as generate_token is not async
    
    // 4. Create Session
    // We use a longer TTL (e.g., 24 hours) for email verification links
    let tenant_id = user.tenant_id;
    let (session, _) = otp_service.create_session(
        tenant_id,
        email.clone(),
        "email".to_string(),
        DeliveryMethod::Email,
        OtpPurpose::EmailVerification,
        Some(user.id),
        Some(token.clone()), // Explicit token
        Some(1440), // 24 hours
    )?; // Removed .await as create_session is not async

    // 5. Save to DB
    let token_hash = otp_service.hash_otp(&token)?;
    otp_repo.create_session(&session, &token_hash).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;

    // 6. Send Email
    // Construct Link: https://api.upflame.com/auth/verify/email?token=...&verification_id=...
    // In production, this base URL should be configurable per tenant or env
    let base_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let link = format!("{}/auth/verify/email?token={}&verification_id={}", base_url, token, session.id);
    
    otp_delivery.send_verification_email(&email, &link).await.map_err(|e| ApiError::from(e))?;

    Ok((
        StatusCode::OK,
        Json(VerificationResponse {
            verification_id: session.id,
            sent_to: email,
            method: "magic_link".to_string(),
            expires_at: session.expires_at.to_rfc3339(),
        })
    ))
}

/// GET /auth/verify/email
/// Magic Link Handler
pub async fn verify_email_link(
    State(identity_service): State<Arc<IdentityService>>,
    State(otp_service): State<Arc<OtpService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    Query(query): Query<MagicLinkQuery>,
) -> Result<impl IntoResponse, ApiError> {
    
    // 1. Fetch Session
    let (session, token_hash): (auth_core::services::otp_service::OtpSession, String) = otp_repo.find_by_id(query.verification_id).await?
        .ok_or(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Invalid }))?;

    // 2. Validate
    if otp_service.is_expired(&session) { // is_expired returns bool directly
        return Err(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Expired }));
    }
    if otp_service.is_verified(&session) { // is_verified is not fallible and doesn't need map_err
         // Already verified, return success idempotent
        return Ok("Email already verified".to_string());
    }

    // 3. Verify Token
    if !otp_service.verify_otp(&query.token, &token_hash).map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))? {
        otp_repo.increment_attempts(session.id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
        return Err(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Invalid }));
    }

    // 4. Update User Status
    if let Some(user_id) = session.user_id {
        identity_service.mark_email_verified(user_id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
        otp_repo.mark_verified(session.id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
        Ok("Email verified successfully! You can now close this window.".to_string())
    } else {
        Err(ApiError::new(auth_core::error::AuthError::InternalError))
    }
}

// ============================================================================
// Phone Verification
// ============================================================================

/// POST /auth/verify/phone/send
/// Sends SMS OTP
pub async fn send_phone_verification(
    State(identity_service): State<Arc<IdentityService>>,
    State(otp_service): State<Arc<OtpService>>,
    State(otp_delivery): State<Arc<OtpDeliveryService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<SendPhoneVerificationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    
    let user = identity_service.get_user(payload.user_id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
    let phone = payload.phone.or(user.phone).ok_or(ApiError::new(auth_core::error::AuthError::ValidationError { message: "No phone number".to_string() }))?;
    
    // Rate Limit
    let limit_key = format!("verify_phone:{}", user.id);
    let is_allowed: bool = rate_limiter.check_limit(&limit_key, "phone_verification").await.map_err(|e| ApiError::new(auth_core::error::AuthError::InternalError))?;
    if !is_allowed {
         return Err(ApiError::new(auth_core::error::AuthError::RateLimitExceeded { limit: 3, window: "1 hour".to_string() }));
    }

    // Generate numeric OTP (6 digits)
    let tenant_id = user.tenant_id;
    let (session, otp): (auth_core::services::otp_service::OtpSession, String) = otp_service.create_session(
        tenant_id,
        phone.clone(),
        "phone".to_string(),
        DeliveryMethod::Sms,
        OtpPurpose::PhoneVerification,
        Some(user.id),
        None, // auto-generate
        Some(10), // 10 minutes TTL
    )?; // Removed .await as create_session is not async

    let otp_hash = otp_service.hash_otp(&otp)?;
    otp_repo.create_session(&session, &otp_hash).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
    
    otp_delivery.send_phone_otp(&phone, &otp).await.map_err(|e| ApiError::from(e))?;

    Ok((
        StatusCode::OK,
        Json(VerificationResponse {
            verification_id: session.id,
            sent_to: phone,
            method: "sms".to_string(),
            expires_at: session.expires_at.to_rfc3339(),
        })
    ))
}

/// POST /auth/verify/phone/confirm
/// Verifies SMS OTP
pub async fn confirm_phone_verification(
    State(identity_service): State<Arc<IdentityService>>,
    State(otp_service): State<Arc<OtpService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    Json(payload): Json<ConfirmVerificationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    
    let (session, otp_hash): (auth_core::services::otp_service::OtpSession, String) = otp_repo.find_by_id(payload.verification_id).await?
        .ok_or(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Invalid }))?;
        
    if otp_service.is_expired(&session) { // is_expired returns bool directly
         return Err(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Expired }));
    }

    if !otp_service.verify_otp(&payload.code, &otp_hash).map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))? {
        otp_repo.increment_attempts(session.id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
        return Err(ApiError::new(auth_core::error::AuthError::TokenError { kind: TokenErrorKind::Invalid }));
    }

    identity_service.mark_phone_verified(payload.user_id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;
    otp_repo.mark_verified(session.id).await.map_err(|_| ApiError::new(auth_core::error::AuthError::InternalError))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Phone verified successfully"
        }))
    ))
}
