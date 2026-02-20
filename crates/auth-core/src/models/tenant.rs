//! Tenant model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Tenant {
    pub id: Uuid,
    pub organization_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    #[validate(url)]
    pub custom_domain: Option<String>,
    pub branding_config: serde_json::Value,
    pub auth_config: serde_json::Value,
    pub compliance_config: serde_json::Value,
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TenantStatus {
    #[default]
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTenantRequest {
    pub organization_id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    #[validate(url)]
    pub custom_domain: Option<String>,
    pub branding_config: Option<serde_json::Value>,
    pub auth_config: Option<serde_json::Value>,
    pub compliance_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTenantRequest {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(url)]
    pub custom_domain: Option<String>,
    pub branding_config: Option<serde_json::Value>,
    pub auth_config: Option<serde_json::Value>,
    pub compliance_config: Option<serde_json::Value>,
    pub status: Option<TenantStatus>,
}

impl Tenant {
    /// Check if the tenant is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, TenantStatus::Active)
    }

    /// Get the tenant's domain (custom domain or default)
    pub fn get_domain(&self) -> Option<&str> {
        self.custom_domain.as_deref()
    }

    /// Check if tenant has custom branding configured
    pub fn has_custom_branding(&self) -> bool {
        !self.branding_config.is_null()
            && self
                .branding_config
                .as_object()
                .is_some_and(|obj| !obj.is_empty())
    }

    /// Validate slug format (alphanumeric and hyphens only)
    pub fn is_valid_slug(slug: &str) -> bool {
        slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
    }
}
