//! OTP Repository - Database layer for OTP sessions

use auth_core::error::AuthError;
use auth_core::services::otp_service::{DeliveryMethod, OtpPurpose, OtpSession};
use chrono::{DateTime, Utc};
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;

pub struct OtpRepository {
    pool: Pool<MySql>,
}

impl OtpRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    /// Create new OTP session in database
    pub async fn create_session(
        &self,
        session: &OtpSession,
        otp_hash: &str,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            INSERT INTO otp_sessions (
                id, user_id, tenant_id, identifier_type, identifier,
                otp_hash, delivery_method, purpose, sent_at, expires_at,
                attempts, max_attempts, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.user_id.map(|id| id.to_string()))
        .bind(session.tenant_id.to_string())
        .bind(&session.identifier_type)
        .bind(&session.identifier)
        .bind(otp_hash)
        .bind(match &session.delivery_method {
            DeliveryMethod::Email => "email",
            DeliveryMethod::Sms => "sms",
        })
        .bind(session.purpose.as_str())
        .bind(session.sent_at)
        .bind(session.expires_at)
        .bind(session.attempts as i32)
        .bind(session.max_attempts as i32)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }

    /// Find OTP session by ID
    pub async fn find_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<(OtpSession, String)>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, tenant_id, identifier_type, identifier,
                   otp_hash, delivery_method, purpose, sent_at, expires_at,
                   attempts, max_attempts, verified_at
            FROM otp_sessions
            WHERE id = ?
            "#,
        )
        .bind(session_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        if let Some(row) = row {
            let session = self.row_to_session(row)?;
            let otp_hash: String =
                sqlx::query_scalar("SELECT otp_hash FROM otp_sessions WHERE id = ?")
                    .bind(session_id.to_string())
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AuthError::DatabaseError {
                        message: e.to_string(),
                    })?;

            Ok(Some((session, otp_hash)))
        } else {
            Ok(None)
        }
    }

    /// Increment verification attempts
    pub async fn increment_attempts(&self, session_id: Uuid) -> Result<u32, AuthError> {
        sqlx::query("UPDATE otp_sessions SET attempts = attempts + 1 WHERE id = ?")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

        let attempts: i32 = sqlx::query_scalar("SELECT attempts FROM otp_sessions WHERE id = ?")
            .bind(session_id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(attempts as u32)
    }

    /// Mark session as verified
    pub async fn mark_verified(&self, session_id: Uuid) -> Result<(), AuthError> {
        sqlx::query("UPDATE otp_sessions SET verified_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    /// Count recent OTP requests for rate limiting
    pub async fn count_recent_requests(
        &self,
        identifier: &str,
        tenant_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<i64, AuthError> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM otp_sessions
            WHERE identifier = ? AND tenant_id = ? AND created_at > ?
            "#,
        )
        .bind(identifier)
        .bind(tenant_id.to_string())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(count)
    }

    /// Cleanup expired sessions (run as background job)
    pub async fn cleanup_expired(&self) -> Result<u64, AuthError> {
        let result =
            sqlx::query("DELETE FROM otp_sessions WHERE expires_at < ? OR verified_at IS NOT NULL")
                .bind(Utc::now())
                .execute(&self.pool)
                .await
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?;

        Ok(result.rows_affected())
    }

    fn row_to_session(&self, row: sqlx::mysql::MySqlRow) -> Result<OtpSession, AuthError> {
        let user_id_str: Option<String> =
            row.try_get("user_id")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?;

        let user_id = user_id_str
            .map(|s: String| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;

        let id_str: String = row.try_get("id").map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;
        let id = Uuid::parse_str(&id_str).map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        let tenant_id_str: String =
            row.try_get("tenant_id")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?;
        let tenant_id = Uuid::parse_str(&tenant_id_str).map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        let delivery_str: String =
            row.try_get("delivery_method")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?;
        let delivery_method = match delivery_str.as_str() {
            "email" => DeliveryMethod::Email,
            "sms" => DeliveryMethod::Sms,
            _ => DeliveryMethod::Email,
        };

        let purpose_str: String = row
            .try_get("purpose")
            .map_err(|e| AuthError::DatabaseError {
                message: e.to_string(),
            })?;
        let purpose = match purpose_str.as_str() {
            "registration" => OtpPurpose::Registration,
            "login" => OtpPurpose::Login,
            "verification" => OtpPurpose::EmailVerification,
            "password_reset" => OtpPurpose::PasswordReset,
            _ => OtpPurpose::Login,
        };

        Ok(OtpSession {
            id,
            user_id,
            tenant_id,
            identifier_type: row.try_get("identifier_type").map_err(|e| {
                AuthError::DatabaseError {
                    message: e.to_string(),
                }
            })?,
            identifier: row
                .try_get("identifier")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?,
            delivery_method,
            purpose,
            attempts: row
                .try_get::<i32, _>("attempts")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })? as u32,
            max_attempts: row.try_get::<i32, _>("max_attempts").map_err(|e| {
                AuthError::DatabaseError {
                    message: e.to_string(),
                }
            })? as u32,
            sent_at: row
                .try_get("sent_at")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?,
            expires_at: row
                .try_get("expires_at")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?,
            verified_at: row
                .try_get("verified_at")
                .map_err(|e| AuthError::DatabaseError {
                    message: e.to_string(),
                })?,
        })
    }
}
