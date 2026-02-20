use anyhow::Result;
use auth_core::error::AuthError;
use auth_core::models::subscription::TenantSubscription;
use auth_core::services::subscription_service::SubscriptionStore;
use sqlx::{MySql, Pool, Row};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SubscriptionRepository {
    pool: Pool<MySql>,
}

impl SubscriptionRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionStore for SubscriptionRepository {
    async fn create(&self, sub: TenantSubscription) -> Result<TenantSubscription, AuthError> {
        sqlx::query(
            r#"
            INSERT INTO tenant_subscriptions (
                id, tenant_id, plan_id, status, start_date, end_date, current_usage, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(sub.id.to_string())
        .bind(sub.tenant_id.to_string())
        .bind(sub.plan_id.clone())
        .bind(sub.status.clone())
        .bind(sub.start_date)
        .bind(sub.end_date)
        .bind(sub.current_usage.clone())
        .bind(sub.created_at)
        .bind(sub.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;

        Ok(sub)
    }

    async fn get_by_tenant(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<TenantSubscription>, AuthError> {
        // Semantic fix: Manually map row to handle potential UUID/String mismatches with sqlx
        let row = sqlx::query("SELECT * FROM tenant_subscriptions WHERE tenant_id = ? ORDER BY created_at DESC LIMIT 1")
            .bind(tenant_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;

        if let Some(row) = row {
            let id_str: String = row.try_get("id").map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;
            let t_id_str: String =
                row.try_get("tenant_id")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?;

            Ok(Some(TenantSubscription {
                id: Uuid::parse_str(&id_str).unwrap_or_default(),
                tenant_id: Uuid::parse_str(&t_id_str).unwrap_or_default(),
                plan_id: row
                    .try_get("plan_id")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?,
                status: row
                    .try_get("status")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?,
                start_date: row
                    .try_get("start_date")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?,
                end_date: row.try_get("end_date").unwrap_or(None),
                current_usage: row.try_get("current_usage").map_err(|e| {
                    AuthError::DatabaseError {
                        message: e.to_string(),
                    }
                })?,
                created_at: row
                    .try_get("created_at")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?,
                updated_at: row
                    .try_get("updated_at")
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_usage(
        &self,
        tenant_id: Uuid,
        usage: HashMap<String, i64>,
    ) -> Result<(), AuthError> {
        // This replaces the usage JSON. For atomic increments, we'd need a more specific query or a specific table for usage events.
        // For this MVP, we will read-modify-write in the service or just overwrite here.
        // For better concurrency, we assume the service passes the updated map.

        let sub = self
            .get_by_tenant(tenant_id)
            .await?
            .ok_or(AuthError::ValidationError {
                message: "No subscription found".to_string(),
            })?;

        sqlx::query(
            "UPDATE tenant_subscriptions SET current_usage = ?, updated_at = NOW() WHERE id = ?",
        )
        .bind(sqlx::types::Json(usage))
        .bind(sub.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }
}
