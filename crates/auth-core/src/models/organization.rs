//! Organization model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Organization {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(url)]
    pub domain: Option<String>,
    pub status: OrganizationStatus,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum OrganizationStatus {
    #[default]
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(url)]
    pub domain: Option<String>,
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateOrganizationRequest {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(url)]
    pub domain: Option<String>,
    pub settings: Option<serde_json::Value>,
    pub status: Option<OrganizationStatus>,
}

impl Organization {
    /// Check if the organization is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrganizationStatus::Active)
    }

    /// Get the organization's domain
    pub fn get_domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Check if organization has custom settings configured
    pub fn has_custom_settings(&self) -> bool {
        !self.settings.is_null() && self.settings.as_object().is_some_and(|obj| !obj.is_empty())
    }
}

