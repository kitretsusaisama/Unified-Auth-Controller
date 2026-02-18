use uuid::Uuid;
use std::sync::Arc;
use crate::error::AuthError;
use crate::models::{Role, CreateRoleRequest};
use async_trait::async_trait;

#[async_trait]
pub trait RoleStore: Send + Sync {
    async fn create(&self, role: Role) -> Result<Role, AuthError>;
    async fn update(&self, role: Role) -> Result<Role, AuthError>;
    async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<(), AuthError>;
    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Role>, AuthError>;
    async fn list(&self, tenant_id: Uuid) -> Result<Vec<Role>, AuthError>;
    async fn assign_permission(&self, role_id: Uuid, permission_id: Uuid) -> Result<(), AuthError>;
}

pub struct AuthorizationService {
    role_store: Arc<dyn RoleStore>,
}

impl AuthorizationService {
    pub fn new(role_store: Arc<dyn RoleStore>) -> Self {
        Self { role_store }
    }

    pub async fn create_role(&self, tenant_id: Uuid, request: CreateRoleRequest) -> Result<Role, AuthError> {
        // Validate scope

        let role = Role {
            id: Uuid::new_v4(),
            tenant_id,
            name: request.name,
            description: request.description,
            parent_role_id: request.parent_role_id,
            is_system_role: false,
            permissions: request.permissions,
            constraints: request.constraints,
            organization_id: None,
            scope: crate::models::RoleScope::Tenant,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        self.role_store.create(role).await
    }

    // ... other CRUD methods
}
