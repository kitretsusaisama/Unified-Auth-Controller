//! Multi-channel user repository extensions
//! 
//! Additional methods for email/phone identifier support

use super::UserRepository;
use auth_core::models::{User, user::{IdentifierType, PrimaryIdentifier}};
use auth_core::error::AuthError;
use uuid::Uuid;
use sqlx::Row;

impl UserRepository {
    /// Find user by identifier (email or phone)
    /// Auto-detects whether identifier is email or phone
    pub async fn find_by_identifier(
        &self,
        identifier: &str,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        // Detect identifier type
        let is_email = identifier.contains('@');
        
        if is_email {
            self.find_by_email(identifier, tenant_id).await
        } else {
            self.find_by_phone(identifier, tenant_id).await
        }
    }
    
    /// Find user by phone number
    pub async fn find_by_phone(
        &self,
        phone: &str,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT id, identifier_type, primary_identifier, email, email_verified, 
                   email_verified_at, phone, phone_verified, phone_verified_at,
                   password_hash, password_changed_at, failed_login_attempts, 
                   locked_until, last_login_at, last_login_ip, mfa_enabled, 
                   mfa_secret, backup_codes, risk_score, profile_data, preferences, 
                   status, created_at, updated_at, deleted_at
            FROM users 
            WHERE phone = ? AND tenant_id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(phone)
        .bind(tenant_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;
        
        if let Some(row) = row {
            self.row_to_user(row).map(Some)
        } else {
            Ok(None)
        }
    }
    
    /// Find user by either email or phone
    /// Useful when user can login with either
    pub async fn find_by_email_or_phone(
        &self,
        email: Option<&str>,
        phone: Option<&str>,
        tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        if let Some(email) = email {
            if let Some(user) = self.find_by_email(email, tenant_id).await? {
                return Ok(Some(user));
            }
        }
        
        if let Some(phone) = phone {
            return self.find_by_phone(phone, tenant_id).await;
        }
        
        Ok(None)
    }
    
    /// Update phone verification status
    pub async fn mark_phone_verified(
        &self,
        user_id: Uuid,
    ) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE users SET phone_verified = true, phone_verified_at = NOW() WHERE id = ?"
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;
        
        Ok(())
    }
    
    /// Update email verification status
    pub async fn mark_email_verified(
        &self,
        user_id: Uuid,
    ) -> Result<(), AuthError> {
        sqlx::query(
            "UPDATE users SET email_verified = true, email_verified_at = NOW() WHERE id = ?"
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::DatabaseError { message: e.to_string() })?;
        
        Ok(())
    }
    
    /// Helper to convert database row to User model
    fn row_to_user(&self, row: sqlx::mysql::MySqlRow) -> Result<User, AuthError> {
        // TODO: Implement full row mapping with new multi-channel fields
        // This is a placeholder - actual implementation needs all fields
        Err(AuthError::InternalError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Integration tests would go here
    // Requires database connection for testing
}
