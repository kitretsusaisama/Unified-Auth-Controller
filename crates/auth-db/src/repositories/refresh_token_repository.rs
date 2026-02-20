//! Refresh token repository for database operations
//! Part of Task 3.3: Implement Refresh Token System with Family Tracking

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool, Row};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Token not found")]
    TokenNotFound,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token already revoked")]
    TokenRevoked,
    #[error("Token family breach detected")]
    FamilyBreachDetected,
}

#[derive(Debug, Clone)]
pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token_family: Uuid,
    pub token_hash: String,
    pub device_fingerprint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct RefreshTokenRepository {
    pool: Pool<MySql>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    /// Create a new refresh token in the database
    pub async fn create(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        token_family: Uuid,
        token_hash: String,
        device_fingerprint: Option<String>,
        user_agent: Option<String>,
        ip_address: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshTokenRecord, RefreshTokenError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (
                id, user_id, tenant_id, token_family, token_hash,
                device_fingerprint, user_agent, ip_address, expires_at, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(tenant_id.to_string())
        .bind(token_family.to_string())
        .bind(&token_hash)
        .bind(&device_fingerprint)
        .bind(&user_agent)
        .bind(&ip_address)
        .bind(expires_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(RefreshTokenRecord {
            id,
            user_id,
            tenant_id,
            token_family,
            token_hash,
            device_fingerprint,
            user_agent,
            ip_address,
            expires_at,
            revoked_at: None,
            revoked_reason: None,
            created_at: now,
        })
    }

    /// Find a refresh token by its hash
    pub async fn find_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<RefreshTokenRecord, RefreshTokenError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, tenant_id, token_family, token_hash,
                   device_fingerprint, user_agent, ip_address,
                   expires_at, revoked_at, revoked_reason, created_at
            FROM refresh_tokens
            WHERE token_hash = ?
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(RefreshTokenError::TokenNotFound)?;

        self.row_to_record(row)
    }

    /// Find all tokens in a token family
    pub async fn find_by_family(
        &self,
        token_family: Uuid,
    ) -> Result<Vec<RefreshTokenRecord>, RefreshTokenError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, tenant_id, token_family, token_hash,
                   device_fingerprint, user_agent, ip_address,
                   expires_at, revoked_at, revoked_reason, created_at
            FROM refresh_tokens
            WHERE token_family = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(token_family.to_string())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_record(row))
            .collect()
    }

    /// Find all active tokens for a user
    pub async fn find_by_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Vec<RefreshTokenRecord>, RefreshTokenError> {
        let now = Utc::now();
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, tenant_id, token_family, token_hash,
                   device_fingerprint, user_agent, ip_address,
                   expires_at, revoked_at, revoked_reason, created_at
            FROM refresh_tokens
            WHERE user_id = ? AND tenant_id = ?
              AND revoked_at IS NULL
              AND expires_at > ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .bind(tenant_id.to_string())
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_record(row))
            .collect()
    }

    /// Revoke a single refresh token
    pub async fn revoke_token(
        &self,
        token_id: Uuid,
        reason: Option<String>,
    ) -> Result<(), RefreshTokenError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = ?, revoked_reason = ?
            WHERE id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(now)
        .bind(reason)
        .bind(token_id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RefreshTokenError::TokenNotFound);
        }

        Ok(())
    }

    /// Revoke entire token family (for breach detection)
    pub async fn revoke_family(
        &self,
        token_family: Uuid,
        reason: String,
    ) -> Result<u64, RefreshTokenError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = ?, revoked_reason = ?
            WHERE token_family = ? AND revoked_at IS NULL
            "#,
        )
        .bind(now)
        .bind(reason)
        .bind(token_family.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Check if a token is valid (exists, not expired, not revoked)
    pub async fn is_token_valid(&self, token_hash: &str) -> Result<bool, RefreshTokenError> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM refresh_tokens
            WHERE token_hash = ?
              AND revoked_at IS NULL
              AND expires_at > ?
            "#,
        )
        .bind(token_hash)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count > 0)
    }

    /// Clean up expired tokens (for background job)
    pub async fn cleanup_expired(&self) -> Result<u64, RefreshTokenError> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < ?
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Detect if a revoked token is being used (potential breach)
    pub async fn detect_breach(&self, token_hash: &str) -> Result<Option<Uuid>, RefreshTokenError> {
        let row = sqlx::query(
            r#"
            SELECT token_family
            FROM refresh_tokens
            WHERE token_hash = ? AND revoked_at IS NOT NULL
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let family_str: String = row.try_get("token_family")?;
            let family = Uuid::parse_str(&family_str)
                .map_err(|_| RefreshTokenError::DatabaseError(sqlx::Error::ColumnNotFound("token_family".to_string())))?;
            Ok(Some(family))
        } else {
            Ok(None)
        }
    }

    /// Helper to convert database row to RefreshTokenRecord
    fn row_to_record(&self, row: sqlx::mysql::MySqlRow) -> Result<RefreshTokenRecord, RefreshTokenError> {
        let id_str: String = row.try_get("id")?;
        let user_id_str: String = row.try_get("user_id")?;
        let tenant_id_str: String = row.try_get("tenant_id")?;
        let family_str: String = row.try_get("token_family")?;

        Ok(RefreshTokenRecord {
            id: Uuid::parse_str(&id_str)
                .map_err(|_| RefreshTokenError::DatabaseError(sqlx::Error::ColumnNotFound("id".to_string())))?,
            user_id: Uuid::parse_str(&user_id_str)
                .map_err(|_| RefreshTokenError::DatabaseError(sqlx::Error::ColumnNotFound("user_id".to_string())))?,
            tenant_id: Uuid::parse_str(&tenant_id_str)
                .map_err(|_| RefreshTokenError::DatabaseError(sqlx::Error::ColumnNotFound("tenant_id".to_string())))?,
            token_family: Uuid::parse_str(&family_str)
                .map_err(|_| RefreshTokenError::DatabaseError(sqlx::Error::ColumnNotFound("token_family".to_string())))?,
            token_hash: row.try_get("token_hash")?,
            device_fingerprint: row.try_get("device_fingerprint")?,
            user_agent: row.try_get("user_agent")?,
            ip_address: row.try_get("ip_address")?,
            expires_at: row.try_get("expires_at")?,
            revoked_at: row.try_get("revoked_at")?,
            revoked_reason: row.try_get("revoked_reason")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

// Imports for trait implementation
use auth_core::services::RefreshTokenStore;
use auth_core::models::RefreshToken;
use auth_core::error::AuthError;

// ... existing code ...

impl RefreshTokenRepository {
    // ... existing methods ...

    /// Save a fully formed refresh token record (used by TokenEngine)
    pub async fn save(&self, record: RefreshTokenRecord) -> Result<(), RefreshTokenError> {
        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (
                id, user_id, tenant_id, token_family, token_hash,
                device_fingerprint, user_agent, ip_address, 
                expires_at, revoked_at, revoked_reason, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(record.id.to_string())
        .bind(record.user_id.to_string())
        .bind(record.tenant_id.to_string())
        .bind(record.token_family.to_string())
        .bind(&record.token_hash)
        .bind(&record.device_fingerprint)
        .bind(&record.user_agent)
        .bind(&record.ip_address)
        .bind(record.expires_at)
        .bind(record.revoked_at)
        .bind(&record.revoked_reason)
        .bind(record.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl RefreshTokenStore for RefreshTokenRepository {
    async fn create(&self, token: RefreshToken) -> Result<(), AuthError> {
        let record = RefreshTokenRecord {
            id: token.id,
            user_id: token.user_id,
            tenant_id: token.tenant_id,
            token_family: token.token_family,
            token_hash: token.token_hash,
            device_fingerprint: token.device_fingerprint,
            user_agent: token.user_agent,
            ip_address: token.ip_address,
            expires_at: token.expires_at,
            revoked_at: token.revoked_at,
            revoked_reason: token.revoked_reason,
            created_at: token.created_at,
        };

        self.save(record).await.map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthError> {
        match self.find_by_token_hash(hash).await {
            Ok(record) => Ok(Some(RefreshToken {
                id: record.id,
                user_id: record.user_id,
                tenant_id: record.tenant_id,
                token_family: record.token_family,
                token_hash: record.token_hash,
                device_fingerprint: record.device_fingerprint,
                user_agent: record.user_agent,
                ip_address: record.ip_address,
                expires_at: record.expires_at,
                revoked_at: record.revoked_at,
                revoked_reason: record.revoked_reason,
                created_at: record.created_at,
            })),
            Err(RefreshTokenError::TokenNotFound) => Ok(None),
            Err(e) => Err(AuthError::DatabaseError { message: e.to_string() }),
        }
    }

    async fn revoke(&self, token_id: Uuid) -> Result<(), AuthError> {
        self.revoke_token(token_id, Some("Revoked by user/system".to_string()))
            .await
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }

    async fn revoke_family(&self, family_id: Uuid) -> Result<(), AuthError> {
        self.revoke_family(family_id, "Family revocation".to_string())
            .await
            .map(|_| ())
            .map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }
}
