use anyhow::Result;
use auth_core::error::AuthError;
use auth_core::models::{Role, UpdateRoleRequest};
use auth_core::services::role_service::RoleStore;
use sqlx::{MySql, Pool, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct RoleRepository {
    pool: Pool<MySql>,
}

impl RoleRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RoleStore for RoleRepository {
    async fn create(&self, role: Role) -> Result<Role, AuthError> {
        // Using sqlx::query instead of query! explicitly to avoid macro type issues with Json types
        sqlx::query(
            r#"
            INSERT INTO roles (id, tenant_id, name, description, parent_role_id, is_system_role, permissions, constraints, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(role.id.to_string())
        .bind(role.tenant_id.to_string())
        .bind(role.name.clone())
        .bind(role.description.clone())
        .bind(role.parent_role_id.map(|u| u.to_string()))
        .bind(role.is_system_role)
        .bind(role.permissions.clone())
        .bind(role.constraints.clone())
        .bind(role.created_at)
        .bind(role.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;

        Ok(role)
    }

    async fn update(&self, id: Uuid, req: UpdateRoleRequest) -> Result<Role, AuthError> {
        let mut current_role = self.find_by_id(id).await?
            .ok_or(AuthError::ValidationError { message: "Role not found".to_string() })?;

        if let Some(name) = req.name { current_role.name = name; }
        if let Some(desc) = req.description { current_role.description = Some(desc); }
        if let Some(parent) = req.parent_role_id { current_role.parent_role_id = Some(parent); }
        if let Some(perms) = req.permissions { current_role.permissions = sqlx::types::Json(perms); }
        if let Some(cons) = req.constraints { current_role.constraints = sqlx::types::Json(cons); }
        current_role.updated_at = Some(chrono::Utc::now());

        sqlx::query(
            r#"
            UPDATE roles
            SET name=?, description=?, parent_role_id=?, permissions=?, constraints=?, updated_at=?
            WHERE id=?
            "#
        )
        .bind(current_role.name.clone())
        .bind(current_role.description.clone())
        .bind(current_role.parent_role_id.map(|u| u.to_string()))
        .bind(current_role.permissions.clone())
        .bind(current_role.constraints.clone())
        .bind(current_role.updated_at)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;

        Ok(current_role)
    }

    async fn delete(&self, id: Uuid) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM roles WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, AuthError> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }

    async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Role>, AuthError> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE tenant_id = ?")
            .bind(tenant_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }

    async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<Role>, AuthError> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }
}
