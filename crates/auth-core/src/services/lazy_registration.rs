//! Lazy Registration Service
//!
//! Handles "Just-in-Time" account creation when a user authenticates
//! with a valid credential (e.g., Firebase, Social, OTP) but does not
//! have an account in the system.

use uuid::Uuid;
use std::sync::Arc;
use crate::models::user::{User, IdentifierType, PrimaryIdentifier};
use crate::services::identity::IdentityService;
use crate::error::AuthError;
use serde_json::json;

pub struct LazyRegistrationService {
    identity_service: Arc<IdentityService>,
    // In a real app, inject TenantRepository to check configs
}

impl LazyRegistrationService {
    pub fn new(identity_service: Arc<IdentityService>) -> Self {
        Self { identity_service }
    }

    /// Handle a login attempt that might require lazy registration
    ///
    /// 1. Check if user exists
    /// 2. If yes, return user
    /// 3. If no, check tenant config
    /// 4. If allowed, create user (Lazy)
    /// 5. Return new user
    pub async fn get_or_create_user(
        &self,
        tenant_id: Uuid,
        identifier: &str,
        identifier_type: IdentifierType,
    ) -> Result<(User, bool), AuthError> {
        // 1. Try to find existing user
        let existing_user = self.identity_service
            .find_user_by_identifier(tenant_id, identifier)
            .await?;

        if let Some(user) = existing_user {
            return Ok((user, false)); // false = was not created
        }

        // 2. User not found - Check if we should lazy register
        // TODO: Load Tenant config here. Assuming TRUE for implementation prototype.
        let allow_lazy = true;

        if !allow_lazy {
            return Err(AuthError::UserNotFound);
        }

        // 3. Create the user "lazily"
        // We set a flag or status indicating profile is incomplete
        let is_email = matches!(identifier_type, IdentifierType::Email);

        let mut profile_data = json!({
            "registration_method": "lazy",
            "source": "auto_creation"
        });

        // Register user via IdentityService (which we need to extend or use existing create)
        // We'll calculate a random password or set none for passwordless users

        // This is a simplified call - in reality we go through CreateUserRequest
        let new_user = self.identity_service.create_lazy_user(
            tenant_id,
            identifier,
            identifier_type,
        ).await?;

        Ok((new_user, true)) // true = was newly created
    }
}
