use auth_core::models::User;
use auth_core::models::user::{UserStatus, CreateUserRequest, UpdateUserRequest};
use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json;
use sqlx::Row;

use auth_core::services::identity::UserStore;
use auth_core::error::AuthError;
use async_trait::async_trait;

#[async_trait]
impl UserStore for UserRepository {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, AuthError> {
        self.find_by_email(email, tenant_id).await.map_err(AuthError::from)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
        self.find_by_id(id).await.map_err(AuthError::from)
    }

    async fn create(&self, user: CreateUserRequest, password_hash: String, tenant_id: Uuid) -> Result<User, AuthError> {
        self.create(user, password_hash, tenant_id).await.map_err(AuthError::from)
    }

    async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), AuthError> {
        self.update_status(id, status).await.map_err(AuthError::from)
    }

    async fn increment_failed_attempts(&self, id: Uuid) -> Result<u32, AuthError> {
        self.increment_failed_attempts(id).await.map_err(AuthError::from)
    }

    async fn reset_failed_attempts(&self, id: Uuid) -> Result<(), AuthError> {
        self.reset_failed_attempts(id).await.map_err(AuthError::from)
    }

    async fn record_login(&self, id: Uuid, ip: Option<String>) -> Result<(), AuthError> {
        self.record_login(id, ip).await.map_err(AuthError::from)
    }
}

#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }



    pub async fn create(&self, request: CreateUserRequest, password_hash: String, tenant_id: Uuid) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let status = UserStatus::PendingVerification;
        let status_str = serde_json::to_string(&status).unwrap_or_else(|_| "\"PendingVerification\"".to_string());
        
        let profile = serde_json::to_value(&request.profile_data).unwrap_or(serde_json::json!({}));

        // 1. INSERT
        sqlx::query(
            r#"
            INSERT INTO users (
                id, tenant_id, email, password_hash, status, 
                created_at, updated_at, email_verified, phone_verified,
                failed_login_attempts, risk_score, mfa_enabled,
                profile_data, preferences
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, false, false, 0, 0.0, false, ?, '{}')
            "#,
        )
        .bind(id.to_string())
        .bind(tenant_id.to_string())
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&status_str)
        .bind(now)
        .bind(now)
        .bind(&profile)
        .execute(&self.pool)
        .await?;

        // 2. FETCH
        self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn find_by_email(&self, email: &str, tenant_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, email, email_verified, phone, phone_verified, password_hash, password_changed_at, failed_login_attempts, locked_until, last_login_at, last_login_ip, mfa_enabled, mfa_secret, backup_codes, risk_score, profile_data, preferences, status, created_at, updated_at, deleted_at
            FROM users 
            WHERE email = ? AND tenant_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(email)
        .bind(tenant_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
         let row = sqlx::query(
            r#"
            SELECT id, email, email_verified, phone, phone_verified, password_hash, password_changed_at, failed_login_attempts, locked_until, last_login_at, last_login_ip, mfa_enabled, mfa_secret, backup_codes, risk_score, profile_data, preferences, status, created_at, updated_at, deleted_at
            FROM users 
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.map_row(row)?))
        } else {
            Ok(None)
        }
    }

    fn map_row(&self, row: sqlx::mysql::MySqlRow) -> Result<User, sqlx::Error> {
        let status_str: String = row.try_get("status")?;
        let status: UserStatus = serde_json::from_str(&status_str).unwrap_or(UserStatus::PendingVerification);
        
        let id_str: String = row.try_get("id")?;
        
        Ok(User {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            email: row.try_get("email")?,
            email_verified: row.try_get("email_verified")?, // maps boolean or tinyint
            phone: row.try_get("phone")?,
            phone_verified: row.try_get("phone_verified")?,
            password_hash: Some(row.try_get("password_hash")?),
            password_changed_at: row.try_get("password_changed_at")?,
            failed_login_attempts: row.try_get::<i32, _>("failed_login_attempts").unwrap_or(0) as u32,
            locked_until: row.try_get("locked_until")?,
            last_login_at: row.try_get("last_login_at")?,
            last_login_ip: row.try_get("last_login_ip")?,
            mfa_enabled: row.try_get("mfa_enabled")?,
            mfa_secret: row.try_get("mfa_secret")?,
            backup_codes: row.try_get("backup_codes").map(|v: serde_json::Value| serde_json::from_value(v).unwrap_or_default()).ok(),
            risk_score: row.try_get::<f32, _>("risk_score").unwrap_or(0.0),
            profile_data: row.try_get::<serde_json::Value, _>("profile_data").unwrap_or(serde_json::json!({})),
            preferences: row.try_get::<serde_json::Value, _>("preferences").unwrap_or(serde_json::json!({})),
            status,
            created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()), 
            updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
            deleted_at: row.try_get("deleted_at")?,
        })
    }




    pub async fn update_status(&self, id: Uuid, status: UserStatus) -> Result<(), sqlx::Error> {
        let status_str = serde_json::to_string(&status).unwrap();
        sqlx::query!(
            "UPDATE users SET status = ?, updated_at = ? WHERE id = ?",
            status_str,
            Utc::now(),
            id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn increment_failed_attempts(&self, id: Uuid) -> Result<u32, sqlx::Error> {
        let now = Utc::now();
        // Assuming max attempts 5 logic is handling in service, here we just increment
        // If > 5, we might set locked_until.
         sqlx::query!(
            "UPDATE users SET failed_login_attempts = failed_login_attempts + 1, updated_at = ? WHERE id = ?",
            now,
            id.to_string()
        )
        .execute(&self.pool)
        .await?;

        // Return new count?
        let count = sqlx::query_scalar!("SELECT failed_login_attempts FROM users WHERE id = ?", id.to_string())
            .fetch_one(&self.pool)
            .await?;
        Ok(count.unwrap_or(0) as u32)
    }

    pub async fn reset_failed_attempts(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, updated_at = ? WHERE id = ?",
            Utc::now(),
            id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    pub async fn record_login(&self, id: Uuid, ip: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET last_login_at = ?, last_login_ip = ?, failed_login_attempts = 0, locked_until = NULL, updated_at = ? WHERE id = ?",
            Utc::now(),
            ip,
            Utc::now(),
            id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
