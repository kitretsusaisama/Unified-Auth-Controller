//! Phone number validation utilities

use regex::Regex;
use std::sync::OnceLock;

static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Validate and normalize phone number to E.164 format
pub fn normalize_phone(phone: &str) -> Result<String, String> {
    // Remove all non-digit characters except +
    let cleaned: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    // E.164 format validation: +[country code][number]
    // Min: +1234567 (7 digits), Max: +123456789012345 (15 digits)
    let regex = PHONE_REGEX.get_or_init(|| Regex::new(r"^\+[1-9]\d{6,14}$").unwrap());

    if regex.is_match(&cleaned) {
        Ok(cleaned)
    } else {
        Err(format!("Invalid phone number format: {}", phone))
    }
}

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), String> {
    // Use validator crate's email validation
    use validator::ValidateEmail;

    if email.validate_email() {
        Ok(())
    } else {
        Err(format!("Invalid email format: {}", email))
    }
}

/// Detect identifier type from string
pub fn detect_identifier_type(identifier: &str) -> IdentifierType {
    if identifier.starts_with('+') || identifier.chars().all(|c| c.is_ascii_digit()) {
        IdentifierType::Phone
    } else if identifier.contains('@') {
        IdentifierType::Email
    } else {
        // Default to email for ambiguous cases
        IdentifierType::Email
    }
}

use super::user::IdentifierType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_phone_valid() {
        assert_eq!(normalize_phone("+14155552671").unwrap(), "+14155552671");
        assert_eq!(
            normalize_phone("+1 (415) 555-2671").unwrap(),
            "+14155552671"
        );
        assert_eq!(normalize_phone("+91 98765 43210").unwrap(), "+919876543210");
    }

    #[test]
    fn test_normalize_phone_invalid() {
        assert!(normalize_phone("123456").is_err()); // Too short
        assert!(normalize_phone("+12345678901234567").is_err()); // Too long
        assert!(normalize_phone("not-a-phone").is_err());
    }

    #[test]
    fn test_detect_identifier() {
        assert!(matches!(
            detect_identifier_type("+14155552671"),
            IdentifierType::Phone
        ));
        assert!(matches!(
            detect_identifier_type("user@example.com"),
            IdentifierType::Email
        ));
    }
}
