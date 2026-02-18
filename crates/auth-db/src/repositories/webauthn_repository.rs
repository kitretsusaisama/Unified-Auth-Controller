use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use uuid::Uuid;
use auth_core::services::webauthn_service::{Passkey, WebauthnStore};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyRecord {
    pub id: String, // Credential ID (Base64)
    pub user_id: Uuid,
    pub passkey_json: String, // Serialized Passkey object
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct WebauthnRepository {
    pool: Pool<MySql>,
}

impl WebauthnRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WebauthnStore for WebauthnRepository {
    async fn save_passkey(&self, user_id: Uuid, passkey: &Passkey) -> anyhow::Result<()> {
        let passkey_json = serde_json::to_string(passkey).unwrap(); 
        
        sqlx::query(
            r#"
            INSERT INTO passkeys (id, user_id, passkey_json, created_at)
            VALUES (?, ?, ?, NOW())
            "#
        )
        .bind(passkey.cred_id().to_string())
        .bind(user_id.to_string())
        .bind(passkey_json)
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?; // Convert sqlx::Error to anyhow::Error

        Ok(())
    }
}

