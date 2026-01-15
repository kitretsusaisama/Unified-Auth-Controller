//! Password policy model and configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordPolicyConfig {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>, // None for global policy
    pub name: String,
    pub description: Option<String>,
    pub policy: PasswordPolicyRules,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyRules {
    // Length requirements
    pub min_length: usize,
    pub max_length: usize,
    
    // Character requirements
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub min_special_chars: usize,
    
    // Complexity requirements
    pub min_character_classes: usize, // How many different types of chars required
    pub disallow_common_passwords: bool,
    pub disallow_personal_info: bool,
    pub disallow_repeated_chars: bool,
    pub disallow_sequential_chars: bool,
    
    // Aging and history
    pub max_age_days: Option<u32>,
    pub history_count: usize, // Number of previous passwords to remember
    pub min_age_hours: Option<u32>, // Minimum time before password can be changed again
    
    // Lockout policy
    pub lockout_threshold: u32,
    pub lockout_duration_minutes: u32,
    pub lockout_reset_time_minutes: Option<u32>, // Time after which failed attempts reset
    
    // Advanced features
    pub require_mfa_for_privileged: bool,
    pub password_strength_meter: bool,
    pub custom_dictionary: Vec<String>, // Additional forbidden words
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePasswordPolicyRequest {
    pub tenant_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub description: Option<String>,
    pub policy: PasswordPolicyRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePasswordPolicyRequest {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub policy: Option<PasswordPolicyRules>,
    pub is_active: Option<bool>,
}

impl Default for PasswordPolicyRules {
    fn default() -> Self {
        Self {
            // Length requirements - enterprise grade
            min_length: 12,
            max_length: 128,
            
            // Character requirements
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_special_chars: 2,
            
            // Complexity requirements
            min_character_classes: 3,
            disallow_common_passwords: true,
            disallow_personal_info: true,
            disallow_repeated_chars: true,
            disallow_sequential_chars: true,
            
            // Aging and history
            max_age_days: Some(90),
            history_count: 12,
            min_age_hours: Some(24),
            
            // Lockout policy
            lockout_threshold: 5,
            lockout_duration_minutes: 30,
            lockout_reset_time_minutes: Some(60),
            
            // Advanced features
            require_mfa_for_privileged: true,
            password_strength_meter: true,
            custom_dictionary: Vec::new(),
        }
    }
}

impl PasswordPolicyConfig {
    /// Create a new password policy configuration
    pub fn new(tenant_id: Option<Uuid>, name: String, policy: PasswordPolicyRules) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name,
            description: None,
            policy,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if this policy is more restrictive than another
    pub fn is_more_restrictive_than(&self, other: &PasswordPolicyRules) -> bool {
        self.policy.min_length >= other.min_length
            && self.policy.min_special_chars >= other.min_special_chars
            && self.policy.lockout_threshold <= other.lockout_threshold
            && self.policy.history_count >= other.history_count
    }

    /// Get effective policy by merging with defaults
    pub fn get_effective_policy(&self) -> PasswordPolicyRules {
        // In a real implementation, this would merge tenant-specific policies
        // with organization and global defaults
        self.policy.clone()
    }
}

/// Predefined policy templates for common use cases
pub struct PasswordPolicyTemplates;

impl PasswordPolicyTemplates {
    /// Basic policy for low-security environments
    pub fn basic() -> PasswordPolicyRules {
        PasswordPolicyRules {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: false,
            min_special_chars: 0,
            min_character_classes: 3,
            disallow_common_passwords: true,
            disallow_personal_info: false,
            disallow_repeated_chars: false,
            disallow_sequential_chars: false,
            max_age_days: Some(180),
            history_count: 3,
            min_age_hours: None,
            lockout_threshold: 10,
            lockout_duration_minutes: 15,
            lockout_reset_time_minutes: Some(60),
            require_mfa_for_privileged: false,
            password_strength_meter: true,
            custom_dictionary: Vec::new(),
        }
    }

    /// Enterprise policy for business environments
    pub fn enterprise() -> PasswordPolicyRules {
        PasswordPolicyRules::default()
    }

    /// High-security policy for sensitive environments
    pub fn high_security() -> PasswordPolicyRules {
        PasswordPolicyRules {
            min_length: 16,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_special_chars: 3,
            min_character_classes: 4,
            disallow_common_passwords: true,
            disallow_personal_info: true,
            disallow_repeated_chars: true,
            disallow_sequential_chars: true,
            max_age_days: Some(60),
            history_count: 24,
            min_age_hours: Some(48),
            lockout_threshold: 3,
            lockout_duration_minutes: 60,
            lockout_reset_time_minutes: Some(30),
            require_mfa_for_privileged: true,
            password_strength_meter: true,
            custom_dictionary: Vec::new(),
        }
    }

    /// Compliance policy for regulated industries (HIPAA, PCI-DSS, etc.)
    pub fn compliance() -> PasswordPolicyRules {
        PasswordPolicyRules {
            min_length: 14,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_special_chars: 2,
            min_character_classes: 4,
            disallow_common_passwords: true,
            disallow_personal_info: true,
            disallow_repeated_chars: true,
            disallow_sequential_chars: true,
            max_age_days: Some(90),
            history_count: 12,
            min_age_hours: Some(24),
            lockout_threshold: 5,
            lockout_duration_minutes: 30,
            lockout_reset_time_minutes: Some(60),
            require_mfa_for_privileged: true,
            password_strength_meter: true,
            custom_dictionary: Vec::new(),
        }
    }
}