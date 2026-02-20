//! Permission model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Permission {
    pub id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub code: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub resource_type: Option<String>,
    pub action: Option<String>,
    pub conditions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub granted: bool,
    pub conditions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
