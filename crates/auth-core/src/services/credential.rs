//! Credential management service

use crate::error::AuthError;
use crate::models::{PasswordPolicyRules, PasswordPolicyTemplates};
// TODO: Implement PasswordHasher in auth_crypto
// use auth_crypto::hashing::PasswordHasher;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Temporary stub until auth_crypto::hashing is implemented
#[derive(Debug, Clone)]
struct PasswordHasher;
impl PasswordHasher {
    fn new() -> Self { Self }
    fn hash_password(&self, _password: &str) -> Result<String, String> { Ok("stubbed".to_string()) }
    fn verify_password(&self, _password: &str, _hash: &str) -> Result<bool, String> { Ok(false) }
}

#[derive(Debug, Clone)]
pub struct CredentialService {
    password_hasher: PasswordHasher,
    policy: PasswordPolicyRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CredentialRequest {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    #[validate(length(min = 1))]
    pub password: String,
    pub current_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub strength_score: u8, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrengthResult {
    pub score: u8, // 0-100
    pub feedback: Vec<String>,
    pub estimated_crack_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHistoryEntry {
    pub user_id: Uuid,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl CredentialService {
    pub fn new(policy: Option<PasswordPolicyRules>) -> Self {
        Self {
            password_hasher: PasswordHasher::new(),
            policy: policy.unwrap_or_default(),
        }
    }

    /// Create service with predefined policy template
    pub fn with_template(template: &str) -> Self {
        let policy = match template {
            "basic" => PasswordPolicyTemplates::basic(),
            "enterprise" => PasswordPolicyTemplates::enterprise(),
            "high_security" => PasswordPolicyTemplates::high_security(),
            "compliance" => PasswordPolicyTemplates::compliance(),
            _ => PasswordPolicyRules::default(),
        };
        Self::new(Some(policy))
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        self.password_hasher
            .hash_password(password)
            .map_err(|e| AuthError::CredentialError {
                message: format!("Failed to hash password: {}", e),
            })
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        self.password_hasher
            .verify_password(password, hash)
            .map_err(|e| AuthError::CredentialError {
                message: format!("Failed to verify password: {}", e),
            })
    }

    /// Validate password against policy
    pub fn validate_password(&self, password: &str) -> CredentialValidationResult {
        let mut errors = Vec::new();
        let mut score = 0u8;

        // Length validation
        if password.len() < self.policy.min_length {
            errors.push(format!("Password must be at least {} characters long", self.policy.min_length));
        } else if password.len() >= self.policy.min_length {
            score += 20;
        }

        if password.len() > self.policy.max_length {
            errors.push(format!("Password must not exceed {} characters", self.policy.max_length));
        }

        // Character requirements
        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_numbers = password.chars().any(|c| c.is_ascii_digit());
        let special_char_count = password.chars().filter(|c| !c.is_alphanumeric()).count();

        let mut character_classes = 0;
        if has_uppercase { character_classes += 1; }
        if has_lowercase { character_classes += 1; }
        if has_numbers { character_classes += 1; }
        if special_char_count > 0 { character_classes += 1; }

        if self.policy.require_uppercase && !has_uppercase {
            errors.push("Password must contain at least one uppercase letter".to_string());
        } else if has_uppercase {
            score += 15;
        }

        if self.policy.require_lowercase && !has_lowercase {
            errors.push("Password must contain at least one lowercase letter".to_string());
        } else if has_lowercase {
            score += 15;
        }

        if self.policy.require_numbers && !has_numbers {
            errors.push("Password must contain at least one number".to_string());
        } else if has_numbers {
            score += 15;
        }

        if self.policy.require_special_chars && special_char_count < self.policy.min_special_chars {
            errors.push(format!(
                "Password must contain at least {} special character(s)",
                self.policy.min_special_chars
            ));
        } else if special_char_count >= self.policy.min_special_chars {
            score += 15;
        }

        // Character class diversity
        if character_classes < self.policy.min_character_classes {
            errors.push(format!(
                "Password must contain at least {} different types of characters",
                self.policy.min_character_classes
            ));
        }

        // Pattern checks
        if self.policy.disallow_common_passwords && self.has_common_patterns(password) {
            errors.push("Password contains common patterns and may be easily guessed".to_string());
            score = score.saturating_sub(20);
        }

        if self.policy.disallow_repeated_chars && self.has_repeated_chars(password) {
            errors.push("Password contains too many repeated characters".to_string());
            score = score.saturating_sub(10);
        }

        if self.policy.disallow_sequential_chars && self.has_sequential_chars(password) {
            errors.push("Password contains sequential characters (abc, 123)".to_string());
            score = score.saturating_sub(10);
        }

        // Custom dictionary check
        if !self.policy.custom_dictionary.is_empty() && self.contains_custom_words(password) {
            errors.push("Password contains forbidden words".to_string());
            score = score.saturating_sub(15);
        }

        // Additional strength bonuses
        if password.len() >= 16 {
            score += 10;
        }
        if password.len() >= 20 {
            score += 5;
        }

        CredentialValidationResult {
            is_valid: errors.is_empty(),
            errors,
            strength_score: score.min(100),
        }
    }

    /// Calculate password strength with detailed feedback
    pub fn calculate_password_strength(&self, password: &str) -> PasswordStrengthResult {
        let validation = self.validate_password(password);
        let mut feedback = Vec::new();

        // Provide constructive feedback
        if password.len() < self.policy.min_length + 4 {
            feedback.push(format!("Consider using a longer password ({}+ characters)", self.policy.min_length + 4));
        }

        if !password.chars().any(|c| !c.is_alphanumeric()) {
            feedback.push("Add special characters for better security".to_string());
        }

        if self.has_repeated_chars(password) {
            feedback.push("Avoid repeated characters".to_string());
        }

        if self.has_sequential_chars(password) {
            feedback.push("Avoid sequential characters (abc, 123)".to_string());
        }

        if self.has_common_patterns(password) {
            feedback.push("Avoid common words and patterns".to_string());
        }

        // Estimate crack time based on score
        let crack_time = match validation.strength_score {
            0..=20 => "Seconds to minutes",
            21..=40 => "Minutes to hours",
            41..=60 => "Hours to days",
            61..=80 => "Days to months",
            _ => "Months to years", // 81-100 and any other values
        };

        PasswordStrengthResult {
            score: validation.strength_score,
            feedback,
            estimated_crack_time: crack_time.to_string(),
        }
    }

    /// Check if password change is required based on age
    pub fn is_password_change_required(&self, password_changed_at: Option<DateTime<Utc>>) -> bool {
        if let (Some(max_age_days), Some(changed_at)) = (self.policy.max_age_days, password_changed_at) {
            let max_age = Duration::days(max_age_days as i64);
            Utc::now() - changed_at > max_age
        } else {
            false
        }
    }

    /// Check if password can be changed (minimum age check)
    pub fn can_change_password(&self, password_changed_at: Option<DateTime<Utc>>) -> bool {
        if let (Some(min_age_hours), Some(changed_at)) = (self.policy.min_age_hours, password_changed_at) {
            let min_age = Duration::hours(min_age_hours as i64);
            Utc::now() - changed_at >= min_age
        } else {
            true
        }
    }

    /// Check if account should be locked based on failed attempts
    pub fn should_lock_account(&self, failed_attempts: u32) -> bool {
        failed_attempts >= self.policy.lockout_threshold
    }

    /// Calculate when account should be unlocked
    pub fn calculate_unlock_time(&self) -> DateTime<Utc> {
        Utc::now() + Duration::minutes(self.policy.lockout_duration_minutes as i64)
    }

    /// Check if password is in history (would need to be implemented with database)
    pub fn is_password_in_history(&self, password: &str, history: &[PasswordHistoryEntry]) -> Result<bool, AuthError> {
        for entry in history.iter().take(self.policy.history_count) {
            if self.verify_password(password, &entry.password_hash)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Validate credential request
    pub fn validate_credential_request(&self, request: &CredentialRequest) -> Result<(), AuthError> {
        // Validate the request structure
        request.validate().map_err(|e| AuthError::ValidationError {
            message: format!("Invalid credential request: {}", e),
        })?;

        // Validate password policy
        let validation = self.validate_password(&request.password);
        if !validation.is_valid {
            return Err(AuthError::PasswordPolicyViolation {
                errors: validation.errors,
            });
        }

        Ok(())
    }

    /// Check for common password patterns
    fn has_common_patterns(&self, password: &str) -> bool {
        let common_patterns = [
            "password", "123456", "qwerty", "admin", "letmein",
            "welcome", "monkey", "dragon", "master", "shadow",
            "login", "pass", "root", "user", "test", "guest",
            "demo", "temp", "default", "changeme", "secret"
        ];

        let lower_password = password.to_lowercase();
        common_patterns.iter().any(|&pattern| lower_password.contains(pattern))
    }

    /// Check for repeated characters
    fn has_repeated_chars(&self, password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        let mut repeat_count = 0;

        for i in 0..chars.len().saturating_sub(1) {
            if chars[i] == chars[i + 1] {
                repeat_count += 1;
                if repeat_count >= 2 { // Allow some repetition but not too much
                    return true;
                }
            } else {
                repeat_count = 0;
            }
        }
        false
    }

    /// Check for sequential characters
    fn has_sequential_chars(&self, password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let c1 = chars[i] as u8;
            let c2 = chars[i + 1] as u8;
            let c3 = chars[i + 2] as u8;

            if (c2 == c1 + 1 && c3 == c2 + 1) || (c2 == c1 - 1 && c3 == c2 - 1) {
                return true;
            }
        }
        false
    }

    /// Check for custom dictionary words
    fn contains_custom_words(&self, password: &str) -> bool {
        let lower_password = password.to_lowercase();
        self.policy.custom_dictionary.iter()
            .any(|word| lower_password.contains(&word.to_lowercase()))
    }

    /// Get password policy
    pub fn get_policy(&self) -> &PasswordPolicyRules {
        &self.policy
    }

    /// Update password policy
    pub fn update_policy(&mut self, policy: PasswordPolicyRules) {
        self.policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        let service = CredentialService::new(None);

        // Test weak password
        let result = service.validate_password("weak");
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());

        // Test strong password that meets all default policy requirements
        let result = service.validate_password("StrongP@ssw0rd246!Extra");
        assert!(result.is_valid, "Errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
        assert!(result.strength_score > 80);
    }

    #[test]
    fn test_password_hashing() {
        let service = CredentialService::new(None);
        let password = "TestPassword246!";

        let hash = service.hash_password(password).unwrap();
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_common_patterns() {
        let service = CredentialService::new(None);

        assert!(service.has_common_patterns("password123"));
        assert!(service.has_common_patterns("MyPassword"));
        assert!(!service.has_common_patterns("MySecureP@ssw0rd"));
    }

    #[test]
    fn test_lockout_logic() {
        let service = CredentialService::new(None);

        assert!(!service.should_lock_account(4));
        assert!(service.should_lock_account(5));
        assert!(service.should_lock_account(10));
    }

    #[test]
    fn test_policy_templates() {
        let basic = CredentialService::with_template("basic");
        let enterprise = CredentialService::with_template("enterprise");
        let high_security = CredentialService::with_template("high_security");

        assert_eq!(basic.get_policy().min_length, 8);
        assert_eq!(enterprise.get_policy().min_length, 12);
        assert_eq!(high_security.get_policy().min_length, 16);
    }

    #[test]
    fn test_character_class_validation() {
        let service = CredentialService::new(None);

        // Test password with insufficient character classes
        let result = service.validate_password("alllowercase");
        assert!(!result.is_valid);

        // Test password with sufficient character classes and length
        let result = service.validate_password("MixedCase246!@#Extra");
        assert!(result.is_valid, "Errors: {:?}", result.errors);
    }
}