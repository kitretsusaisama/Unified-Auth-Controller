use crate::error::AuthError;
use crate::models::{CreateRoleRequest, Role, RoleScope, UpdateRoleRequest};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait RoleStore: Send + Sync {
    async fn create(&self, role: Role) -> Result<Role, AuthError>;
    async fn update(&self, id: Uuid, req: UpdateRoleRequest) -> Result<Role, AuthError>;
    async fn delete(&self, id: Uuid) -> Result<(), AuthError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, AuthError>;
    async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Role>, AuthError>;
    async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<Role>, AuthError>;
}

pub struct RoleService {
    store: Arc<dyn RoleStore>,
}

impl RoleService {
    pub fn new(store: Arc<dyn RoleStore>) -> Self {
        Self { store }
    }

    pub async fn create_role(
        &self,
        tenant_id: Uuid,
        req: CreateRoleRequest,
    ) -> Result<Role, AuthError> {
        // Check if role name exists in tenant
        if self
            .store
            .find_by_name(tenant_id, &req.name)
            .await?
            .is_some()
        {
            return Err(AuthError::ValidationError {
                message: "Role with this name already exists".to_string(),
            });
        }

        let role = Role {
            id: Uuid::new_v4(),
            tenant_id,
            name: req.name,
            description: req.description,
            parent_role_id: req.parent_role_id,
            is_system_role: false,
            permissions: req.permissions,
            constraints: req.constraints,
            organization_id: None, // Assuming tenant level for now, can extend Request to include org_id
            scope: RoleScope::Tenant,
            metadata: None,
            created_at: chrono::Utc::now(),
            updated_at: None,
        };

        self.store.create(role).await
    }

    // TODO: Add other service methods (update, delete, inheritance resolution)
}
