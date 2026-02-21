//! Multi-channel user repository extensions
//!
//! Additional methods for email/phone identifier support

use super::user_repository::UserRepository;
use auth_core::error::AuthError;
use auth_core::models::User;
use uuid::Uuid;

impl UserRepository {
    /// Find user by either email or phone
    /// Useful when user can login with either
    pub async fn find_by_email_or_phone(
        &self,
        email: Option<&str>,
        phone: Option<&str>,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        if let Some(email) = email {
            if let Some(user) = self.find_by_email(email, tenant_id).await.map_err(|e| {
                AuthError::DatabaseError {
                    message: e.to_string(),
                }
            })? {
                return Ok(Some(user));
            }
        }

        if let Some(phone) = phone {
            return self.find_by_phone(phone, tenant_id).await.map_err(|e| {
                AuthError::DatabaseError {
                    message: e.to_string(),
                }
            });
        }

        Ok(None)
    }

    /// Update phone verification status
    pub async fn mark_phone_verified(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE users SET phone_verified = true, phone_verified_at = NOW() WHERE id = ?",
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }

    /// Update email verification status
    pub async fn mark_email_verified(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE id = ?",
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }

    /// Helper to convert database row to User model
    #[allow(dead_code)]
    fn row_to_user(&self, row: sqlx::mysql::MySqlRow) -> Result<User, AuthError> {
        self.map_row(row).map_err(|e| AuthError::DatabaseError {
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here
    // Requires database connection for testing
}
