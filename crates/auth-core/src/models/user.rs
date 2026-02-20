//! User model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Identifier type for user authentication
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(rename_all = "snake_case")]
pub enum IdentifierType {
    Email,
    Phone,
    Both,
}

/// Primary identifier for login
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(rename_all = "snake_case")]
pub enum PrimaryIdentifier {
    Email,
    Phone,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    // Multi-channel identifier fields
    pub identifier_type: IdentifierType,
    pub primary_identifier: PrimaryIdentifier,

    #[validate(email)]
    pub email: Option<String>,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,

    pub phone: Option<String>,
    pub phone_verified: bool,
    pub phone_verified_at: Option<DateTime<Utc>>,

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

impl Default for User {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            identifier_type: IdentifierType::Email,
            primary_identifier: PrimaryIdentifier::Email,
            email: None,
            email_verified: false,
            email_verified_at: None,
            phone: None,
            phone_verified: false,
            phone_verified_at: None,
            password_hash: None,
            password_changed_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            last_login_at: None,
            last_login_ip: None,
            mfa_enabled: false,
            mfa_secret: None,
            backup_codes: None,
            risk_score: 0.0,
            profile_data: serde_json::json!({}),
            preferences: serde_json::json!({}),
            status: UserStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(rename_all = "snake_case")]
#[derive(Default)]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
    #[default]
    PendingVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateUserRequest {
    /// Type of identifier being used
    pub identifier_type: IdentifierType,

    #[validate(email)]
    pub email: Option<String>,

    pub phone: Option<String>,

    /// Primary identifier for login (email or phone)
    pub primary_identifier: Option<PrimaryIdentifier>,

    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,

    pub profile_data: Option<serde_json::Value>,

    /// Require verification after registration
    pub require_verification: Option<bool>,
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

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Suspended => write!(f, "suspended"),
            UserStatus::Deleted => write!(f, "deleted"),
            UserStatus::PendingVerification => write!(f, "pending_verification"),
        }
    }
}
