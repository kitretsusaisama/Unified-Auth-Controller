use anyhow::Result;
use auth_core::error::AuthError;
use auth_core::models::Session;
use auth_core::services::session_service::SessionStore;
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;

pub struct SessionRepository {
    pool: Pool<MySql>,
}

impl SessionRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionStore for SessionRepository {
    async fn create(&self, session: Session) -> Result<Session, AuthError> {
        // Using sqlx::query explicitly to avoid potential macro type complexities with Option<String> / Uuid
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, user_id, tenant_id, session_token, device_fingerprint, 
                user_agent, ip_address, risk_score, last_activity, expires_at, created_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(session.id.to_string())
        .bind(session.user_id.to_string())
        .bind(session.tenant_id.to_string())
        .bind(session.session_token.clone())
        .bind(session.device_fingerprint.clone())
        .bind(session.user_agent.clone())
        .bind(session.ip_address.clone())
        .bind(session.risk_score)
        .bind(session.last_activity)
        .bind(session.expires_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(session)
    }

    async fn get(&self, session_token: &str) -> Result<Option<Session>, AuthError> {
        sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE session_token = ?")
            .bind(session_token)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    async fn delete(&self, session_token: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE session_token = ?")
            .bind(session_token)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_by_user(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
