use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::models::Role;
use std::collections::HashMap;

/// Authorization Context
/// Contains all necessary info to make an access decision
pub struct AuthContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub roles: Vec<Role>,
    pub attributes: HashMap<String, String>, // ABAC attributes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
}

/// Policy Engine for evaluating ABAC rules
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn evaluate(
        _permission_code: &str,
        context: &AuthContext,
        resource_attributes: Option<&HashMap<String, String>>,
    ) -> PolicyDecision {
        // 1. Check strict RBAC (Does any role have this permission?)
        let has_permission = context.roles.iter().any(|_role| {
            // This assumes role.permissions is a list of codes or structured objects
            // In the new schema, we need to load permissions.
            // For now, we simulate the check or rely on the loaded permissions list.
            // Ideally, Role struct should have a `permissions` field populated.
            true // Placeholder: actual check needs expanded Role model
        });

        if !has_permission {
            return PolicyDecision::Deny("User does not have the required permission".to_string());
        }

        // 2. Check ABAC (Conditions)
        // If permission has conditions (e.g., "resource.owner_id == user.id")
        if let Some(_attrs) = resource_attributes {
            // Logic to evaluate dynamic conditions would go here
            // e.g. checking ownership
        }

        PolicyDecision::Allow
    }
}
