use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<Uuid>,
    pub is_system_role: bool,
    pub permissions: sqlx::types::Json<Vec<String>>, // Storing codes/IDs
    pub constraints: sqlx::types::Json<HashMap<String, String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub constraints: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub parent_role_id: Option<Uuid>,
    pub permissions: Option<Vec<String>>,
    pub constraints: Option<HashMap<String, String>>,
}
