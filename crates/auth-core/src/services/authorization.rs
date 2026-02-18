//! Authorization service for RBAC and ABAC

use crate::error::AuthError;
use crate::models::{Role, CreateRoleRequest};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait AuthorizationProvider: Send + Sync {
    async fn authorize(&self, context: AuthzContext) -> Result<AuthzDecision, AuthError>;
    async fn create_role(&self, role: CreateRoleRequest) -> Result<Role, AuthError>;
    async fn assign_role(&self, assignment: RoleAssignment) -> Result<(), AuthError>;
    async fn evaluate_policy(&self, policy: Policy, context: Context) -> Result<Decision, AuthError>;
}

#[derive(Debug, Clone)]
pub struct AuthzContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub resource: String,
    pub action: String,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct AuthzDecision {
    pub allowed: bool,
    pub reason: String,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RoleAssignment {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role_id: Uuid,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub rules: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub permit: bool,
    pub obligations: Vec<String>,
}

pub struct AuthorizationEngine {
    // Implementation will be added in later tasks
}

impl AuthorizationEngine {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for AuthorizationEngine {
    fn default() -> Self {
        Self::new()
    }
}
