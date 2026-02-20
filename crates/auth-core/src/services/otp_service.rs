//! OTP (One-Time Password) Service
//!
//! Handles generation, storage, and verification of OTPs for:
//! - Passwordless login
//! - Registration verification
//! - Email/Phone verification
//! - Password reset

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OtpError {
    #[error("OTP generation failed")]
    GenerationFailed,

    #[error("OTP storage failed: {0}")]
    StorageFailed(String),

    #[error("OTP not found")]
    NotFound,

    #[error("OTP expired")]
    Expired,

    #[error("Invalid OTP")]
    Invalid,

    #[error("Maximum attempts exceeded")]
    MaxAttemptsExceeded,

    #[error("OTP already verified")]
    AlreadyVerified,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, Clone)]
pub enum OtpPurpose {
    Registration,
    Login,
    EmailVerification,
    PhoneVerification,
    PasswordReset,
}

impl OtpPurpose {
    pub fn as_str(&self) -> &str {
        match self {
            OtpPurpose::Registration => "registration",
            OtpPurpose::Login => "login",
            OtpPurpose::EmailVerification => "verification",
            OtpPurpose::PhoneVerification => "verification",
            OtpPurpose::PasswordReset => "password_reset",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Numeric,
    Alphanumeric,
}

#[derive(Debug, Clone)]
pub enum DeliveryMethod {
    Email,
    Sms,
}

#[derive(Debug, Clone)]
pub struct OtpSession {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub identifier_type: String,
    pub identifier: String,
    pub delivery_method: DeliveryMethod,
    pub purpose: OtpPurpose,
    pub attempts: u32,
    pub max_attempts: u32,
    pub sent_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

pub struct OtpService {
    // Configuration
    pub default_length: usize,
    pub default_ttl_minutes: i64,
    pub max_attempts: u32,
}

impl OtpService {
    pub fn new() -> Self {
        Self {
            default_length: 6,
            default_ttl_minutes: 10,
            max_attempts: 5,
        }
    }

    /// Generate a random token based on type and length
    pub fn generate_token(&self, token_type: TokenType, length: usize) -> String {
        let mut rng = rand::thread_rng();
        match token_type {
            TokenType::Numeric => {
                // Ensure we don't overflow u32 for length > 9, though for OTPs usually < 10
                // For longer tokens, we might want string manipulation
                if length > 9 {
                    (0..length)
                        .map(|_| rng.gen_range(0..10).to_string())
                        .collect()
                } else {
                    let min = 10_u32.pow((length as u32) - 1);
                    let max = 10_u32.pow(length as u32) - 1;
                    let num: u32 = rng.gen_range(min..=max);
                    num.to_string()
                }
            }
            TokenType::Alphanumeric => rng
                .sample_iter(&Alphanumeric)
                .take(length)
                .map(char::from)
                .collect(),
        }
    }

    // Legacy helper for backward compatibility / convenience
    pub fn generate_otp(&self) -> String {
        self.generate_token(TokenType::Numeric, self.default_length)
    }

    /// Hash OTP for secure storage
    pub fn hash_otp(&self, otp: &str) -> Result<String, OtpError> {
        hash(otp, DEFAULT_COST).map_err(|_| OtpError::GenerationFailed)
    }

    /// Verify OTP against hash
    pub fn verify_otp(&self, otp: &str, hash: &str) -> Result<bool, OtpError> {
        verify(otp, hash).map_err(|_| OtpError::Invalid)
    }

    /// Verify TOTP code against a secret
    pub fn verify_totp(&self, secret_str: &str, code: &str) -> Result<bool, OtpError> {
        let secret = Secret::Encoded(secret_str.to_string())
            .to_bytes()
            .map_err(|_| OtpError::Invalid)?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            Some("AuthPlatform".to_string()),
            "user".to_string(),
        )
        .map_err(|_| OtpError::GenerationFailed)?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    /// Create new OTP session
    /// If `explicit_token` is provided, it is used. Otherwise, one is generated.
    #[allow(clippy::too_many_arguments)]
    pub fn create_session(
        &self,
        tenant_id: Uuid,
        identifier: String,
        identifier_type: String,
        delivery_method: DeliveryMethod,
        purpose: OtpPurpose,
        user_id: Option<Uuid>,
        explicit_token: Option<String>,
        ttl_minutes: Option<i64>,
    ) -> Result<(OtpSession, String), OtpError> {
        let otp = explicit_token.unwrap_or_else(|| self.generate_otp());
        let _otp_hash = self.hash_otp(&otp)?;

        let now = Utc::now();
        let ttl = ttl_minutes.unwrap_or(self.default_ttl_minutes);
        let expires_at = now + Duration::minutes(ttl);

        let session = OtpSession {
            id: Uuid::new_v4(),
            user_id,
            tenant_id,
            identifier_type,
            identifier,
            delivery_method,
            purpose,
            attempts: 0,
            max_attempts: self.max_attempts,
            sent_at: now,
            expires_at,
            verified_at: None,
        };

        Ok((session, otp))
    }

    /// Check if session is expired
    pub fn is_expired(&self, session: &OtpSession) -> bool {
        session.expires_at < Utc::now()
    }

    /// Check if session is already verified
    pub fn is_verified(&self, session: &OtpSession) -> bool {
        session.verified_at.is_some()
    }

    /// Check if max attempts exceeded
    pub fn is_max_attempts_exceeded(&self, session: &OtpSession) -> bool {
        session.attempts >= session.max_attempts
    }
}

impl Default for OtpService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_numeric_token() {
        let service = OtpService::new();
        let token = service.generate_token(TokenType::Numeric, 6);
        assert_eq!(token.len(), 6);
        assert!(token.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_generate_alphanumeric_token() {
        let service = OtpService::new();
        let token = service.generate_token(TokenType::Alphanumeric, 32);
        assert_eq!(token.len(), 32);
    }

    #[test]
    fn test_create_session_defaults() {
        let service = OtpService::new();
        let (session, otp) = service
            .create_session(
                Uuid::new_v4(),
                "test@example.com".to_string(),
                "email".to_string(),
                DeliveryMethod::Email,
                OtpPurpose::Login,
                None,
                None,
                None,
            )
            .unwrap();

        assert_eq!(otp.len(), 6);
        assert_eq!(session.attempts, 0);
    }

    #[test]
    fn test_create_session_explicit() {
        let service = OtpService::new();
        let custom_token = "ABC123XYZ";
        let (session, otp) = service
            .create_session(
                Uuid::new_v4(),
                "test@example.com".to_string(),
                "email".to_string(),
                DeliveryMethod::Email,
                OtpPurpose::Login,
                None,
                Some(custom_token.to_string()),
                Some(60),
            )
            .unwrap();

        assert_eq!(otp, custom_token);
        assert!(session.expires_at > Utc::now() + Duration::minutes(59));
    }
}
