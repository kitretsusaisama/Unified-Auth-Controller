//! User model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct User {
    pub id: Uuid,
    #[validate(email)]
    pub email: String,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub phone_verified: bool,
    pub password_hash: Option<String>,
    pub password_changed_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub last_login_ip: Option<String>,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub backup_codes: Option<Vec<String>>,
    pub risk_score: f32,
    pub profile_data: serde_json::Value,
    pub preferences: serde_json::Value,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
    PendingVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    pub phone: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
    pub profile_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserRequest {
    pub id: Uuid,
    #[validate(email)]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub profile_data: Option<serde_json::Value>,
    pub preferences: Option<serde_json::Value>,
}

impl User {
    /// Check if the user is locked due to failed login attempts
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            locked_until > Utc::now()
        } else {
            false
        }
    }

    /// Check if the user account is active and can authenticate
    pub fn can_authenticate(&self) -> bool {
        matches!(self.status, UserStatus::Active) && !self.is_locked()
    }

    /// Check if the user's email is verified
    pub fn is_email_verified(&self) -> bool {
        self.email_verified
    }

    /// Get the user's risk score
    pub fn get_risk_score(&self) -> f32 {
        self.risk_score.clamp(0.0, 1.0)
    }
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::PendingVerification
    }
}