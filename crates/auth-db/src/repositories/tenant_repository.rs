use anyhow::Result;
use auth_core::error::AuthError;
use auth_core::models::tenant::{Tenant, TenantStatus};
use auth_core::services::tenant_service::TenantStore;
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;

pub struct TenantRepository {
    pool: Pool<MySql>,
}

impl TenantRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TenantStore for TenantRepository {
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>, AuthError> {
        let row = sqlx::query("SELECT * FROM tenants WHERE id = ?")
            .bind(tenant_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

        if let Some(row) = row {
            let id_str: String = row.try_get("id").map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;
            let org_id_str: String = row.try_get("organization_id").map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

            let status_str: String = row.try_get("status").map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

            let status = match status_str.as_str() {
                "active" => TenantStatus::Active,
                "suspended" => TenantStatus::Suspended,
                "deleted" => TenantStatus::Deleted,
                _ => TenantStatus::Active,
            };

            Ok(Some(Tenant {
                id: Uuid::parse_str(&id_str).map_err(|e| AuthError::DatabaseError { message: format!("Invalid UUID for id: {}", e) })?,
                organization_id: Uuid::parse_str(&org_id_str).map_err(|e| AuthError::DatabaseError { message: format!("Invalid UUID for org_id: {}", e) })?,
                name: row.try_get("name").map_err(|e| AuthError::DatabaseError { message: e.to_string() })?,
                slug: row.try_get("slug").map_err(|e| AuthError::DatabaseError { message: e.to_string() })?,
                custom_domain: row.try_get("custom_domain").map_err(|e| AuthError::DatabaseError { message: e.to_string() })?,
                branding_config: row.try_get::<Option<sqlx::types::Json<serde_json::Value>>, _>("branding_config")
                    .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?
                    .map(|json| json.0)
                    .unwrap_or(serde_json::Value::Null),
                auth_config: row.try_get::<Option<sqlx::types::Json<serde_json::Value>>, _>("auth_config")
                    .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?
                    .map(|json| json.0)
                    .unwrap_or(serde_json::Value::Null),
                compliance_config: row.try_get::<Option<sqlx::types::Json<serde_json::Value>>, _>("compliance_config")
                    .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?
                    .map(|json| json.0)
                    .unwrap_or(serde_json::Value::Null),
                status,
                created_at: row.try_get("created_at").map_err(|e| AuthError::DatabaseError { message: e.to_string() })?,
                updated_at: row.try_get("updated_at").map_err(|e| AuthError::DatabaseError { message: e.to_string() })?,
            }))
        } else {
            Ok(None)
        }
    }
}
