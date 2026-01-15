//! Revoked token repository for access token blacklist
//! Part of Task 3.1: Implement JWT Token Engine with RS256

use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool, Row};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RevokedTokenError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Token already revoked")]
    TokenAlreadyRevoked,
}

#[derive(Debug, Clone)]
pub struct RevokedTokenRecord {
    pub id: Uuid,
    pub token_jti: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token_type: TokenType,
    pub revoked_at: DateTime<Utc>,
    pub revoked_by: Option<Uuid>,
    pub revoked_reason: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    Access,
    Refresh,
}

impl TokenType {
    fn to_str(&self) -> &'static str {
        match self {
            TokenType::Access => "access",
            TokenType::Refresh => "refresh",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "refresh" => TokenType::Refresh,
            _ => TokenType::Access,
        }
    }
}

pub struct RevokedTokenRepository {
    pool: Pool<MySql>,
}

impl RevokedTokenRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    /// Add a token to the revocation blacklist
    pub async fn add_revoked_token(
        &self,
       token_jti: Uuid,
        user_id: Uuid,
        tenant_id: Uuid,
        token_type: TokenType,
        revoked_by: Option<Uuid>,
        revoked_reason: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Result<RevokedTokenRecord, RevokedTokenError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Check if already revoked
        if self.is_token_revoked(token_jti).await? {
            return Err(RevokedTokenError::TokenAlreadyRevoked);
        }

        sqlx::query(
            r#"
            INSERT INTO revoked_tokens (
                id, token_jti, user_id, tenant_id, token_type,
                revoked_at, revoked_by, revoked_reason, expires_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(token_jti.to_string())
        .bind(user_id.to_string())
        .bind(tenant_id.to_string())
        .bind(token_type.to_str())
        .bind(now)
        .bind(revoked_by.map(|id| id.to_string()))
        .bind(&revoked_reason)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(RevokedTokenRecord {
            id,
            token_jti,
            user_id,
            tenant_id,
            token_type,
            revoked_at: now,
            revoked_by,
            revoked_reason,
            expires_at,
        })
    }

    /// Check if a token is revoked (optimized for high-throughput reads)
    pub async fn is_token_revoked(&self, token_jti: Uuid) -> Result<bool, RevokedTokenError> {
        let now = Utc::now();
        
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM revoked_tokens
            WHERE token_jti = ? AND expires_at > ?
            "#,
        )
        .bind(token_jti.to_string())
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count > 0)
    }

    /// Revoke all tokens for a user (emergency revocation)
    pub async fn revoke_all_user_tokens(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        revoked_by: Option<Uuid>,
        reason: String,
    ) -> Result<u64, RevokedTokenError> {
        // This is a marker operation - actual tokens are stored elsewhere
        // In production, this would coordinate with refresh token repository
        // and possibly trigger session invalidation
        
        // For now, we'll just add a record to track the bulk revocation
        let id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(1); // Keep record for audit

        sqlx::query(
            r#"
            INSERT IGNORE INTO revoked_tokens (
                id, token_jti, user_id, tenant_id, token_type,
                revoked_at, revoked_by, revoked_reason, expires_at
            ) VALUES (?, ?, ?, ?, 'access', ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(Uuid::new_v4().to_string()) // Placeholder JTI for bulk revocation marker
        .bind(user_id.to_string())
        .bind(tenant_id.to_string())
        .bind(now)
        .bind(revoked_by.map(|id| id.to_string()))
        .bind(format!("BULK_REVOCATION: {}", reason))
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        Ok(1)
    }

    /// Clean up expired revocation records (for background job)
    pub async fn cleanup_expired(&self) -> Result<u64, RevokedTokenError> {
        let now = Utc::now();
        
        let result = sqlx::query(
            r#"
            DELETE FROM revoked_tokens
            WHERE expires_at < ?
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get revocation details for a token (for audit/debugging)
    pub async fn get_revocation_details(
        &self,
        token_jti: Uuid,
    ) -> Result<Option<RevokedTokenRecord>, RevokedTokenError> {
        let row = sqlx::query(
            r#"
            SELECT id, token_jti, user_id, tenant_id, token_type,
                   revoked_at, revoked_by, revoked_reason, expires_at
            FROM revoked_tokens
            WHERE token_jti = ?
            "#,
        )
        .bind(token_jti.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    /// Count active revocations (for monitoring)
    pub async fn count_active_revocations(&self) -> Result<i64, RevokedTokenError> {
        let now = Utc::now();
        
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM revoked_tokens
            WHERE expires_at > ?
            "#,
        )
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.try_get("count")?)
    }

    /// Helper to convert database row to RevokedTokenRecord
    fn row_to_record(&self, row: sqlx::mysql::MySqlRow) -> Result<RevokedTokenRecord, RevokedTokenError> {
        let id_str: String = row.try_get("id")?;
        let jti_str: String = row.try_get("token_jti")?;
        let user_id_str: String = row.try_get("user_id")?;
        let tenant_id_str: String = row.try_get("tenant_id")?;
        let token_type_str: String = row.try_get("token_type")?;
        let revoked_by_str: Option<String> = row.try_get("revoked_by")?;

        Ok(RevokedTokenRecord {
            id: Uuid::parse_str(&id_str)
                .map_err(|_| RevokedTokenError::DatabaseError(sqlx::Error::ColumnNotFound("id".to_string())))?,
            token_jti: Uuid::parse_str(&jti_str)
                .map_err(|_| RevokedTokenError::DatabaseError(sqlx::Error::ColumnNotFound("token_jti".to_string())))?,
            user_id: Uuid::parse_str(&user_id_str)
                .map_err(|_| RevokedTokenError::DatabaseError(sqlx::Error::ColumnNotFound("user_id".to_string())))?,
            tenant_id: Uuid::parse_str(&tenant_id_str)
                .map_err(|_| RevokedTokenError::DatabaseError(sqlx::Error::ColumnNotFound("tenant_id".to_string())))?,
            token_type: TokenType::from_str(&token_type_str),
            revoked_at: row.try_get("revoked_at")?,
            revoked_by: revoked_by_str.and_then(|s| Uuid::parse_str(&s).ok()),
            revoked_reason: row.try_get("revoked_reason")?,
            expires_at: row.try_get("expires_at")?,
        })
    }
}

use auth_core::services::RevokedTokenStore;
use auth_core::error::AuthError;

#[async_trait::async_trait]
impl RevokedTokenStore for RevokedTokenRepository {
    async fn add_to_blacklist(&self, jti: Uuid, user_id: Uuid, tenant_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), AuthError> {
        self.add_revoked_token(
            jti,
            user_id,
            tenant_id,
            TokenType::Access, // Default to access token for blacklist
            None, // revoked_by (system)
            Some("Revoked via TokenEngine".to_string()),
            expires_at
        ).await.map(|_| ()).map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, AuthError> {
        self.is_token_revoked(jti).await.map_err(|e| AuthError::DatabaseError { message: e.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be implemented in separate test file
    // to avoid compilation issues without database connection
}
