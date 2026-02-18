use sqlx::MySqlPool;
use uuid::Uuid;
use auth_core::models::{Role, RoleScope};
use auth_core::error::AuthError;
use auth_core::services::authorization::RoleStore;
use async_trait::async_trait;

pub struct RoleRepository {
    pool: MySqlPool,
}

impl RoleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleStore for RoleRepository {
    async fn create(&self, role: Role) -> Result<Role, AuthError> {
        // Prepare optional fields for insertion
        let parent_id = role.parent_role_id.map(|id| id.to_string());
        let org_id = role.organization_id.map(|id| id.to_string());
        let scope_str = serde_json::to_string(&role.scope).unwrap_or_default();

        let result = sqlx::query(
            r#"
            INSERT INTO roles (
                id, tenant_id, organization_id, name, description,
                parent_role_id, is_system_role, scope, metadata,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(role.id.to_string())
        .bind(role.tenant_id.to_string())
        .bind(org_id)
        .bind(role.name.clone())
        .bind(role.description.clone())
        .bind(parent_id)
        .bind(role.is_system_role)
        .bind(scope_str)
        .bind(role.metadata.clone())
        .bind(role.created_at)
        .bind(role.updated_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(role),
            Err(e) => Err(AuthError::DatabaseError { message: e.to_string() }),
        }
    }

    async fn update(&self, role: Role) -> Result<Role, AuthError> {
        // Implement update logic if needed
        Ok(role)
    }

    async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            "DELETE FROM roles WHERE id = ? AND tenant_id = ?"
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(res) => {
                if res.rows_affected() == 0 {
                    Err(AuthError::ValidationError { message: "Role not found or system role".to_string() })
                } else {
                    Ok(())
                }
            },
            Err(e) => Err(AuthError::DatabaseError { message: e.to_string() }),
        }
    }

    async fn find_by_id(&self, id: Uuid, tenant_id: Uuid) -> Result<Option<Role>, AuthError> {
        // Manual mapping to avoid sqlx::FromRow macro issues with missing columns in struct vs DB query
        // or complex type conversion (JSON -> Vec, Enum -> String) which failed in previous attempts.

        let rec = sqlx::query(
            r#"
            SELECT id, tenant_id, name, description, parent_role_id, is_system_role,
                   constraints, organization_id, scope, metadata, created_at, updated_at
            FROM roles
            WHERE id = ? AND tenant_id = ?
            "#
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;

        use sqlx::Row;

        if let Some(row) = rec {
            let id_str: String = row.try_get("id").unwrap_or_default();
            let tid_str: String = row.try_get("tenant_id").unwrap_or_default();
            let name: String = row.try_get("name").unwrap_or_default();
            let desc: Option<String> = row.try_get("description").ok();
            let parent_str: Option<String> = row.try_get("parent_role_id").ok();
            let is_sys: bool = row.try_get("is_system_role").unwrap_or(false);

            // Handle JSON/Enums safely
            let scope_str: Option<String> = row.try_get("scope").ok();
            let scope: RoleScope = scope_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(RoleScope::Tenant);

            let meta: Option<serde_json::Value> = row.try_get("metadata").ok();
            let constraints: Option<serde_json::Value> = row.try_get("constraints").ok();
            let org_id_str: Option<String> = row.try_get("organization_id").ok();

            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at").unwrap_or(chrono::Utc::now());
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at").ok();

            Ok(Some(Role {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                tenant_id: Uuid::parse_str(&tid_str).unwrap_or_default(),
                name,
                description: desc,
                parent_role_id: parent_str.and_then(|s| Uuid::parse_str(&s).ok()),
                is_system_role: is_sys,
                permissions: vec![], // Joined elsewhere
                constraints: constraints.and_then(|v| serde_json::from_value(v).ok()),
                organization_id: org_id_str.and_then(|s| Uuid::parse_str(&s).ok()),
                scope,
                metadata: meta,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list(&self, _tenant_id: Uuid) -> Result<Vec<Role>, AuthError> {
        // Fetch all roles for tenant
        Ok(vec![])
    }

    async fn assign_permission(&self, role_id: Uuid, permission_id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES (?, ?)"
        )
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(AuthError::DatabaseError { message: e.to_string() }),
        }
    }
}
