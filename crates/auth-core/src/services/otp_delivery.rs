//! OTP Delivery Service
//!
//! Handles delivery of OTPs via:
//! - Firebase Authentication (for phone OTP)
//! - SMTP (for email OTP)
//!
//! Includes circuit breakers and fallback mechanisms

use async_trait::async_trait;
use thiserror::Error;
use std::sync::Arc;
use lettre::{
    Message, SmtpTransport, Transport,
    transport::smtp::authentication::Credentials,
};

#[derive(Debug, Error)]
pub enum DeliveryError {
    #[error("SMS delivery failed: {0}")]
    SmsFailed(String),

    #[error("Email delivery failed: {0}")]
    EmailFailed(String),

    #[error("Circuit breaker open for {0}")]
    CircuitBreakerOpen(String),

    #[error("All delivery methods failed")]
    AllMethodsFailed,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Provider configuration error: {0}")]
    ConfigError(String),
}

/// SMS/OTP Provider trait
#[async_trait]
pub trait OtpProvider: Send + Sync {
    async fn send_otp(&self, to: &str, otp: &str) -> Result<String, DeliveryError>;
}

/// Email Provider trait
#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, DeliveryError>;
}

/// Firebase Authentication OTP Provider
/// Uses Firebase Phone Authentication to send OTPs
pub struct FirebaseOtpProvider {
    project_id: String,
    api_key: String,
    client: reqwest::Client,
}

impl FirebaseOtpProvider {
    pub fn new(project_id: String, api_key: String) -> Self {
        Self {
            project_id,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Send OTP using Firebase Phone Auth
    async fn send_firebase_otp(&self, phone: &str, otp: &str) -> Result<String, DeliveryError> {
        // Firebase REST API endpoint for sending verification code
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:sendVerificationCode?key={}",
            self.api_key
        );

        let body = serde_json::json!({
            "phoneNumber": phone,
            "recaptchaToken": "skip", // For server-side calls
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DeliveryError::SmsFailed(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json()
                .await
                .map_err(|e| DeliveryError::SmsFailed(e.to_string()))?;

            let session_info = result["sessionInfo"]
                .as_str()
                .ok_or_else(|| DeliveryError::SmsFailed("No session info".to_string()))?;

            tracing::info!("Firebase OTP sent successfully to {}", phone);
            Ok(session_info.to_string())
        } else {
            let error = response.text().await.unwrap_or_default();
            Err(DeliveryError::SmsFailed(format!("Firebase error: {}", error)))
        }
    }
}

#[async_trait]
impl OtpProvider for FirebaseOtpProvider {
    async fn send_otp(&self, to: &str, _otp: &str) -> Result<String, DeliveryError> {
        // Firebase sends its own OTP, we don't need to pass our generated one
        // This is just to initiate the Firebase verification flow
        self.send_firebase_otp(to, "").await
    }
}

/// Alternative: Generic SMS Provider (for services like MSG91, Kaleyra, etc.)
pub struct GenericSmsProvider {
    api_url: String,
    api_key: String,
    sender_id: String,
    client: reqwest::Client,
}

impl GenericSmsProvider {
    pub fn new(api_url: String, api_key: String, sender_id: String) -> Self {
        Self {
            api_url,
            api_key,
            sender_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl OtpProvider for GenericSmsProvider {
    async fn send_otp(&self, to: &str, otp: &str) -> Result<String, DeliveryError> {
        let message = format!("Your verification code is: {}. Valid for 10 minutes. Do not share this code.", otp);

        // Generic SMS API call (adapt based on your provider)
        let response = self.client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "to": to,
                "text": message,
                "senderId": self.sender_id,
            }))
            .send()
            .await
            .map_err(|e| DeliveryError::SmsFailed(e.to_string()))?;

        if response.status().is_success() {
            tracing::info!("SMS OTP sent successfully to {}", to);
            Ok(format!("sms-{}", uuid::Uuid::new_v4()))
        } else {
            let error = response.text().await.unwrap_or_default();
            Err(DeliveryError::SmsFailed(error))
        }
    }
}

/// SMTP Email Provider
/// Uses standard SMTP protocol for email delivery
pub struct SmtpEmailProvider {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
}

impl SmtpEmailProvider {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        from_email: String,
        from_name: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        }
    }

    fn build_mailer(&self) -> Result<SmtpTransport, DeliveryError> {
        let creds = Credentials::new(
            self.smtp_username.clone(),
            self.smtp_password.clone(),
        );

        Ok(SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| DeliveryError::EmailFailed(e.to_string()))?
            .port(self.smtp_port)
            .credentials(creds)
            .build())
    }
}

#[async_trait]
impl EmailProvider for SmtpEmailProvider {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, DeliveryError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().unwrap())
            .to(to.parse().map_err(|e| DeliveryError::EmailFailed(format!("Invalid email: {}", e)))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| DeliveryError::EmailFailed(e.to_string()))?;

        let mailer = self.build_mailer()?;

        // Send email synchronously (lettre doesn't have async SMTP yet)
        let result = tokio::task::spawn_blocking(move || {
            mailer.send(&email)
        })
        .await
        .map_err(|e| DeliveryError::EmailFailed(e.to_string()))?
        .map_err(|e| DeliveryError::EmailFailed(e.to_string()))?;

        tracing::info!("Email sent successfully to {}", to);
        Ok(format!("email-{}", uuid::Uuid::new_v4()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Robust Circuit Breaker for external services
pub struct CircuitBreaker {
    failure_threshold: u32,
    failure_count: Arc<tokio::sync::RwLock<u32>>,
    reset_timeout: std::time::Duration,
    last_failure: Arc<tokio::sync::RwLock<Option<std::time::Instant>>>,
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    half_open_success_threshold: u32,
    success_count: Arc<tokio::sync::RwLock<u32>>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout_secs: u64) -> Self {
        Self {
            failure_threshold,
            failure_count: Arc::new(tokio::sync::RwLock::new(0)),
            reset_timeout: std::time::Duration::from_secs(reset_timeout_secs),
            last_failure: Arc::new(tokio::sync::RwLock::new(None)),
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            half_open_success_threshold: 2, // Require 2 successes to close
            success_count: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    pub async fn is_open(&self) -> bool {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => false,
            CircuitState::Open => {
                let last = self.last_failure.read().await;
                if let Some(time) = *last {
                    if time.elapsed() >= self.reset_timeout {
                        *state = CircuitState::HalfOpen;
                         tracing::warn!("Circuit breaker entering Half-Open state");
                        return false; // Allow traffic in Half-Open
                    }
                }
                true
            }
            CircuitState::HalfOpen => false, // Allow traffic
        }
    }

    pub async fn record_failure(&self) {
        let mut base_state = self.state.write().await;
        let mut failures = self.failure_count.write().await;
        *failures += 1;
        *self.last_failure.write().await = Some(std::time::Instant::now());

        match *base_state {
            CircuitState::Closed => {
                if *failures >= self.failure_threshold {
                    *base_state = CircuitState::Open;
                    tracing::error!("Circuit breaker OPENED after {} failures", *failures);
                }
            }
            CircuitState::HalfOpen => {
                *base_state = CircuitState::Open; // Re-open immediately on failure in Half-Open
                tracing::error!("Circuit breaker RE-OPENED (failure in Half-Open)");
            }
            CircuitState::Open => {
                // Already open, just updated timestamp
            }
        }
    }

    pub async fn record_success(&self) {
        let mut base_state = self.state.write().await;

        match *base_state {
            CircuitState::HalfOpen => {
                let mut successes = self.success_count.write().await;
                *successes += 1;

                if *successes >= self.half_open_success_threshold {
                    *base_state = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                    *successes = 0;
                     tracing::info!("Circuit breaker CLOSED (recovered)");
                }
            }
            CircuitState::Closed => {
                // Optional: decrease failure count over time?
                // For now, simpler to just reset on success or time window.
                // We'll reset failure count on success to be forgiving.
                *self.failure_count.write().await = 0;
            }
            _ => {}
        }
    }
}

/// OTP Delivery Service with Firebase and SMTP
pub struct OtpDeliveryService {
    otp_provider: Arc<dyn OtpProvider>,
    email_provider: Arc<dyn EmailProvider>,
    otp_circuit_breaker: Arc<CircuitBreaker>,
    email_circuit_breaker: Arc<CircuitBreaker>,
}

impl OtpDeliveryService {
    pub fn new(
        otp_provider: Arc<dyn OtpProvider>,
        email_provider: Arc<dyn EmailProvider>,
    ) -> Self {
        Self {
            otp_provider,
            email_provider,
            otp_circuit_breaker: Arc::new(CircuitBreaker::new(5, 60)),
            email_circuit_breaker: Arc::new(CircuitBreaker::new(5, 60)),
        }
    }

    /// Send OTP via SMS/Firebase with circuit breaker
    pub async fn send_phone_otp(&self, to: &str, otp: &str) -> Result<String, DeliveryError> {
        if self.otp_circuit_breaker.is_open().await {
            return Err(DeliveryError::CircuitBreakerOpen("OTP Provider".to_string()));
        }

        match self.otp_provider.send_otp(to, otp).await {
            Ok(msg_id) => {
                self.otp_circuit_breaker.record_success().await;
                Ok(msg_id)
            }
            Err(e) => {
                self.otp_circuit_breaker.record_failure().await;
                Err(e)
            }
        }
    }

    /// Send OTP via email with circuit breaker
    pub async fn send_email_otp(&self, to: &str, otp: &str) -> Result<String, DeliveryError> {
        if self.email_circuit_breaker.is_open().await {
            return Err(DeliveryError::CircuitBreakerOpen("Email".to_string()));
        }

        let subject = "Your Verification Code";
        let body = format!(
            "Your verification code is: {}\n\nThis code will expire in 10 minutes.\n\nIf you didn't request this code, please ignore this email.",
            otp
        );

        match self.email_provider.send_email(to, subject, &body).await {
            Ok(msg_id) => {
                self.email_circuit_breaker.record_success().await;
                Ok(msg_id)
            }
            Err(e) => {
                self.email_circuit_breaker.record_failure().await;
                Err(e)
            }
        }
    }

    /// Send OTP with automatic fallback
    pub async fn send_with_fallback(
        &self,
        identifier: &str,
        otp: &str,
        prefer_phone: bool,
    ) -> Result<(String, &str), DeliveryError> {
        if prefer_phone {
            match self.send_phone_otp(identifier, otp).await {
                Ok(id) => return Ok((id, "phone")),
                Err(e) => {
                    tracing::warn!("Phone OTP delivery failed, trying email: {:?}", e);
                }
            }
        }

        match self.send_email_otp(identifier, otp).await {
            Ok(id) => Ok((id, "email")),
            Err(_) => Err(DeliveryError::AllMethodsFailed),
        }
    }
    /// Send verification email (Magic Link)
    pub async fn send_verification_email(
        &self,
        to: &str,
        link: &str
    ) -> Result<String, DeliveryError> {
        if self.email_circuit_breaker.is_open().await {
            return Err(DeliveryError::CircuitBreakerOpen("Email".to_string()));
        }

        let subject = "Verify your email address";
        let body = format!(
            "Please click the link below to verify your email address:\n\n{}\n\nThis link will expire in 24 hours.\n\nIf you didn't request this, please ignore this email.",
            link
        );

        match self.email_provider.send_email(to, subject, &body).await {
            Ok(msg_id) => {
                self.email_circuit_breaker.record_success().await;
                Ok(msg_id)
            }
            Err(e) => {
                self.email_circuit_breaker.record_failure().await;
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockOtpProvider;

    #[async_trait]
    impl OtpProvider for MockOtpProvider {
        async fn send_otp(&self, _to: &str, _otp: &str) -> Result<String, DeliveryError> {
            Ok("mock-otp-id".to_string())
        }
    }

    struct MockEmailProvider;

    #[async_trait]
    impl EmailProvider for MockEmailProvider {
        async fn send_email(
            &self,
            _to: &str,
            _subject: &str,
            _body: &str,
        ) -> Result<String, DeliveryError> {
            Ok("mock-email-id".to_string())
        }
    }

    #[tokio::test]
    async fn test_send_phone_otp() {
        let service = OtpDeliveryService::new(
            Arc::new(MockOtpProvider),
            Arc::new(MockEmailProvider),
        );

        let result = service.send_phone_otp("+14155552671", "123456").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, 60);

        assert!(!breaker.is_open().await);

        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_failure().await;

        assert!(breaker.is_open().await);

        breaker.record_success().await;
        assert!(!breaker.is_open().await);
    }
}
