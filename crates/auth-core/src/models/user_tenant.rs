//! User-Tenant relationship model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTenant {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub status: UserTenantStatus,
    pub joined_at: DateTime<Utc>,
    pub last_accessed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserTenantStatus {
    Active,
    Suspended,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserTenantRequest {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub status: Option<UserTenantStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role_id: Uuid,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
}

impl Default for UserTenantStatus {
    fn default() -> Self {
        Self::Pending
    }
}
