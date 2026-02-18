//! OTP API Handlers
//! 
//! Endpoints for OTP-based authentication:
//! - POST /auth/otp/request - Request OTP
//! - POST /auth/otp/verify - Verify OTP

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use auth_core::services::{
    otp_service::{OtpService, OtpPurpose, DeliveryMethod},
    otp_delivery::OtpDeliveryService,
    rate_limiter::{RateLimiter, identifier_key},
};
use auth_db::repositories::otp_repository::OtpRepository;
use crate::error::ApiError;
use auth_core::error::{AuthError, TokenErrorKind};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OtpRequestPayload {
    /// Email or phone number
    pub identifier: String,
    /// Tenant ID
    pub tenant_id: Uuid,
    /// Purpose: 'registration', 'login', 'verification', 'password_reset'
    pub purpose: String,
    /// Preferred delivery method: 'email' or 'phone'
    #[serde(default)]
    pub delivery_method: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OtpRequestResponse {
    pub session_id: Uuid,
    pub sent_to: String,
    pub delivery_method: String,
    pub expires_at: String,
    pub retry_after_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct OtpVerifyPayload {
    pub session_id: Uuid,
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct OtpVerifyResponse {
    pub verified: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /auth/otp/request
/// 
/// Request OTP for email or phone
pub async fn request_otp(
    State(otp_service): State<Arc<OtpService>>,
    State(otp_delivery): State<Arc<OtpDeliveryService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<OtpRequestPayload>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Rate limiting check
    let identifier_limit_key = identifier_key(&payload.tenant_id, &payload.identifier);
    
    let is_allowed = rate_limiter
        .check_limit(&identifier_limit_key, "otp_request_per_identifier")
        .await
        .map_err(|_e| ApiError::new(AuthError::InternalError))?;
    if !is_allowed {
        return Err(ApiError::new(AuthError::RateLimitExceeded {
            limit: 5,
            window: "15 minutes".to_string(),
        }));
    }
    
    // 2. Determine identifier type and delivery method
    let identifier_type = if payload.identifier.contains('@') {
        "email".to_string()
    } else {
        "phone".to_string()
    };
    
    let delivery_method = match payload.delivery_method.as_deref().unwrap_or(&identifier_type) {
        "email" => DeliveryMethod::Email,
        "phone" | "sms" => DeliveryMethod::Sms,
        _ => DeliveryMethod::Email,
    };
    
    // 3. Parse purpose
    let purpose = match payload.purpose.as_str() {
        "registration" => OtpPurpose::Registration,
        "login" => OtpPurpose::Login,
        "email_verification" => OtpPurpose::EmailVerification,
        "phone_verification" => OtpPurpose::PhoneVerification,
        "password_reset" => OtpPurpose::PasswordReset,
        _ => OtpPurpose::Login,
    };
    
    // 4. Create OTP session
    let (session, otp) = otp_service.create_session(
        payload.tenant_id,
        payload.identifier.clone(),
        identifier_type.clone(), // Clone to avoid moving
        delivery_method.clone(),
        purpose,
        None, // user_id (optional)
        None, // explicit_token
        None, // ttl_minutes
    )?;
    
    // 5. Hash OTP and save to database
    let otp_hash = otp_service.hash_otp(&otp).map_err(|_| ApiError::new(AuthError::InternalError))?;
    otp_repo.create_session(&session, &otp_hash).await.map_err(|_| ApiError::new(AuthError::InternalError))?;
    
    // 6. Send OTP via appropriate channel
    let delivery_channel = match delivery_method {
        DeliveryMethod::Email => {
            otp_delivery.send_email_otp(&payload.identifier, &otp)
                .await
                .map_err(|_| ApiError::new(AuthError::InternalError))?;
            "email"
        }
        DeliveryMethod::Sms => {
            otp_delivery.send_phone_otp(&payload.identifier, &otp)
                .await
                .map_err(|_| ApiError::new(AuthError::InternalError))?;
            "sms"
        }
    };
    
    // 7. Mask identifier in response
    let masked_identifier = if identifier_type == "email" {
        mask_email(&payload.identifier)
    } else {
        mask_phone(&payload.identifier)
    };
    
    // 8. Return response
    Ok((
        StatusCode::OK,
        Json(OtpRequestResponse {
            session_id: session.id,
            sent_to: masked_identifier,
            delivery_method: delivery_channel.to_string(),
            expires_at: session.expires_at.to_rfc3339(),
            retry_after_seconds: 60,
        }),
    ))
}

/// POST /auth/otp/verify
/// 
/// Verify OTP code
pub async fn verify_otp(
    State(otp_service): State<Arc<OtpService>>,
    State(otp_repo): State<Arc<OtpRepository>>,
    State(rate_limiter): State<Arc<RateLimiter>>,
    Json(payload): Json<OtpVerifyPayload>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Rate limiting check (per session)
    let session_key = format!("otp:session:{}", payload.session_id);
    
    let is_allowed = rate_limiter
        .check_limit(&session_key, "otp_verification_per_session")
        .await
        .map_err(|_e| ApiError::new(AuthError::InternalError))?;
    if !is_allowed {
        return Err(ApiError::new(AuthError::TokenError {
            kind: TokenErrorKind::Invalid,
        }));
    }
    
    // 2. Fetch session from database
    let (session, otp_hash) = otp_repo
        .find_by_id(payload.session_id)
        .await?
        .ok_or_else(|| ApiError::new(AuthError::TokenError {
            kind: TokenErrorKind::Invalid,
        }))?;
    
    // 3. Validate session
    if otp_service.is_expired(&session) { // is_expired returns bool directly
        return Err(ApiError::new(AuthError::TokenError {
            kind: TokenErrorKind::Expired,
        }));
    }
    
    if otp_service.is_verified(&session) { // is_verified returns bool directly
        return Ok((
            StatusCode::OK,
            Json(OtpVerifyResponse {
                verified: true,
                message: "OTP already verified".to_string(),
                user_id: session.user_id,
            }),
        ));
    }
    
    if otp_service.is_max_attempts_exceeded(&session) { // is_max_attempts_exceeded returns bool directly
        return Err(ApiError::new(AuthError::TokenError {
            kind: TokenErrorKind::Invalid,
        }));
    }
    
    // 4. Verify OTP
    let is_valid = otp_service.verify_otp(&payload.otp, &otp_hash).map_err(|_| ApiError::new(AuthError::InternalError))?;
    
    if !is_valid {
        // Increment attempts
        otp_repo.increment_attempts(payload.session_id).await.map_err(|_| ApiError::new(AuthError::InternalError))?;
        
        return Ok((
            StatusCode::UNAUTHORIZED,
            Json(OtpVerifyResponse {
                verified: false,
                message: "Invalid OTP code".to_string(),
                user_id: None,
            }),
        ));
    }
    
    // 5. Mark as verified
    otp_repo.mark_verified(payload.session_id).await.map_err(|_| ApiError::new(AuthError::InternalError))?;
    
    // 6. Return success
    Ok((
        StatusCode::OK,
        Json(OtpVerifyResponse {
            verified: true,
            message: "OTP verified successfully".to_string(),
            user_id: session.user_id,
        }),
    ))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() <= 3 {
            format!("{}****{}", &local[..1], domain)
        } else {
            format!("{}****{}{}", &local[..2], &local[local.len()-1..], domain)
        }
    } else {
        email.to_string()
    }
}

fn mask_phone(phone: &str) -> String {
    if phone.len() > 6 {
        format!("{}****{}", &phone[..3], &phone[phone.len()-3..])
    } else {
        phone.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("test@example.com"), "te****t@example.com");
        assert_eq!(mask_email("a@example.com"), "a****@example.com");
    }
    
    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("+14155552671"), "+14****671");
        assert_eq!(mask_phone("12345"), "12345");
    }
}
