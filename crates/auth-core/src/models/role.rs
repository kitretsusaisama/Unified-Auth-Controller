use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RoleScope {
    Global,
    Organization,
    Tenant,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub parent_role_id: Option<Uuid>,
    pub is_system_role: bool,
    // Note: permissions are now managed via role_permissions table,
    // but we keep this for legacy/read compatibility or as a materialized view
    // if needed. However, the new schema uses a join.
    // For the struct to map to the new schema, we should likely drop this or make it Option<Vec<Permission>>
    // if we join. For now, let's keep it but mark as potentially deprecated in usage.
    pub permissions: Vec<String>,
    pub constraints: Option<HashMap<String, String>>,
    pub organization_id: Option<Uuid>,
    pub scope: RoleScope,
    pub metadata: Option<serde_json::Value>,
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
