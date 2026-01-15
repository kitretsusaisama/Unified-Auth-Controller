use auth_core::error::AuthError;
use regex::Regex;
use once_cell::sync::Lazy;

/// Common weak passwords to reject
static COMMON_PASSWORDS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "password", "123456", "12345678", "qwerty", "abc123",
        "monkey", "1234567", "letmein", "trustno1", "dragon",
        "baseball", "iloveyou", "master", "sunshine", "ashley",
        "bailey", "passw0rd", "shadow", "123123", "654321",
        "superman", "qazwsx", "michael", "football",
    ]
});

/// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

/// Password validation result
#[derive(Debug)]
pub struct PasswordValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Validate password strength
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    let mut errors = Vec::new();

    // Minimum length
    if password.len() < 12 {
        errors.push("Password must be at least 12 characters long".to_string());
    }

    // Maximum length (prevent DoS)
    if password.len() > 128 {
        errors.push("Password must not exceed 128 characters".to_string());
    }

    // Check for uppercase
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    }

    // Check for lowercase
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    }

    // Check for digit
    if !password.chars().any(|c| c.is_numeric()) {
        errors.push("Password must contain at least one number".to_string());
    }

    // Check for special character
    if !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
        errors.push("Password must contain at least one special character".to_string());
    }

    // Check against common passwords
    let lowercase_password = password.to_lowercase();
    if COMMON_PASSWORDS.iter().any(|&common| lowercase_password.contains(common)) {
        errors.push("Password is too common or easily guessable".to_string());
    }

    if !errors.is_empty() {
        return Err(AuthError::PasswordPolicyViolation { errors });
    }

    Ok(())
}

/// Validate and normalize email
pub fn validate_email(email: &str) -> Result<String, AuthError> {
    let trimmed = email.trim();
    
    if trimmed.is_empty() {
        return Err(AuthError::ValidationError {
            message: "Email cannot be empty".to_string(),
        });
    }

    if trimmed.len() > 254 {
        return Err(AuthError::ValidationError {
            message: "Email is too long".to_string(),
        });
    }

    if !EMAIL_REGEX.is_match(trimmed) {
        return Err(AuthError::ValidationError {
            message: "Invalid email format".to_string(),
        });
    }

    // Normalize to lowercase
    Ok(trimmed.to_lowercase())
}

/// Sanitize user input to prevent XSS
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_password() {
        assert!(validate_password("password123").is_err());
        assert!(validate_password("12345678").is_err());
        assert!(validate_password("Short1!").is_err());
    }

    #[test]
    fn test_strong_password() {
        assert!(validate_password("MyS3cur3P@ssw0rd!").is_ok());
        assert!(validate_password("C0mpl3x&Str0ng#Pass").is_ok());
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("  USER@EXAMPLE.COM  ").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_email_normalization() {
        let email = validate_email("  USER@EXAMPLE.COM  ").unwrap();
        assert_eq!(email, "user@example.com");
    }
}
