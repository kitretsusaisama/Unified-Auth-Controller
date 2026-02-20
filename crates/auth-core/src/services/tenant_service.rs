use crate::error::AuthError;
use crate::models::tenant::Tenant;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TenantStore: Send + Sync {
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, AuthError>;
}
