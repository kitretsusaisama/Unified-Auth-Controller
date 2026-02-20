//! Configuration validation utilities

use crate::config::AppConfig;
use secrecy::ExposeSecret;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Error)]
pub enum ConfigValidationError {
    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] ValidationErrors),

    #[error("Security validation failed: {message}")]
    SecurityValidationFailed { message: String },

    #[error("Database validation failed: {message}")]
    DatabaseValidationFailed { message: String },

    #[error("Feature validation failed: {message}")]
    FeatureValidationFailed { message: String },
}

pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate_config(config: &AppConfig) -> Result<(), ConfigValidationError> {
        // Basic validation using validator crate
        config.validate()?;

        // Custom security validations
        Self::validate_security_config(config)?;

        // Custom database validations
        Self::validate_database_config(config)?;

        // Custom feature validations
        Self::validate_feature_config(config)?;

        Ok(())
    }

    fn validate_security_config(config: &AppConfig) -> Result<(), ConfigValidationError> {
        let security = &config.security;

        // JWT secret should be strong enough
        if security.jwt_secret.expose_secret().len() < 32 {
            return Err(ConfigValidationError::SecurityValidationFailed {
                message: "JWT secret must be at least 32 characters long".to_string(),
            });
        }

        // JWT expiry should be reasonable
        if security.jwt_expiry_minutes > 60 {
            return Err(ConfigValidationError::SecurityValidationFailed {
                message: "JWT expiry should not exceed 60 minutes for security".to_string(),
            });
        }

        // Password requirements should be reasonable
        if security.password_min_length < 8 {
            return Err(ConfigValidationError::SecurityValidationFailed {
                message: "Password minimum length should be at least 8 characters".to_string(),
            });
        }

        Ok(())
    }

    fn validate_database_config(config: &AppConfig) -> Result<(), ConfigValidationError> {
        let db = &config.database;

        // Connection pool should be reasonable
        if db.max_connections < db.min_connections {
            return Err(ConfigValidationError::DatabaseValidationFailed {
                message: "Max connections must be greater than or equal to min connections"
                    .to_string(),
            });
        }

        if db.max_connections > 1000 {
            return Err(ConfigValidationError::DatabaseValidationFailed {
                message: "Max connections should not exceed 1000 for performance reasons"
                    .to_string(),
            });
        }

        Ok(())
    }

    fn validate_feature_config(config: &AppConfig) -> Result<(), ConfigValidationError> {
        let features = &config.features;

        // Validate feature limits are reasonable
        for (feature, limit) in &features.feature_limits {
            if *limit == 0 {
                return Err(ConfigValidationError::FeatureValidationFailed {
                    message: format!("Feature limit for '{}' cannot be zero", feature),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;

    fn valid_test_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.security.jwt_secret =
            Secret::new("a-very-long-and-secure-jwt-secret-at-least-32-chars".to_string());
        config
    }

    #[test]
    fn test_valid_config() {
        let config = valid_test_config();
        let result = ConfigValidator::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_jwt_secret() {
        let mut config = valid_test_config();
        config.security.jwt_secret = Secret::new("too-short".to_string());

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::SecurityValidationFailed { message }) => {
                assert!(message.contains("JWT secret must be at least 32 characters long"));
            }
            _ => panic!("Expected SecurityValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_jwt_expiry() {
        let mut config = valid_test_config();
        config.security.jwt_expiry_minutes = 61;

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::SecurityValidationFailed { message }) => {
                assert!(message.contains("JWT expiry should not exceed 60 minutes"));
            }
            _ => panic!("Expected SecurityValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_password_length() {
        let mut config = valid_test_config();
        config.security.password_min_length = 7;

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::SecurityValidationFailed { message }) => {
                assert!(message.contains("Password minimum length should be at least 8 characters"));
            }
            _ => panic!("Expected SecurityValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_db_connections() {
        let mut config = valid_test_config();
        config.database.max_connections = 5;
        config.database.min_connections = 10;

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::DatabaseValidationFailed { message }) => {
                assert!(message
                    .contains("Max connections must be greater than or equal to min connections"));
            }
            _ => panic!("Expected DatabaseValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_db_max_connections() {
        let mut config = valid_test_config();
        config.database.max_connections = 1001;

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::DatabaseValidationFailed { message }) => {
                assert!(message.contains("Max connections should not exceed 1000"));
            }
            _ => panic!("Expected DatabaseValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_invalid_feature_limit() {
        let mut config = valid_test_config();
        config
            .features
            .feature_limits
            .insert("test_feature".to_string(), 0);

        let result = ConfigValidator::validate_config(&config);
        match result {
            Err(ConfigValidationError::FeatureValidationFailed { message }) => {
                assert!(message.contains("Feature limit for 'test_feature' cannot be zero"));
            }
            _ => panic!("Expected FeatureValidationFailed error, got {:?}", result),
        }
    }

    #[test]
    fn test_basic_validation() {
        let mut config = valid_test_config();
        config.server.port = 0; // Invalid port (range is 1-65535)

        let result = ConfigValidator::validate_config(&config);
        assert!(matches!(
            result,
            Err(ConfigValidationError::ValidationFailed(_))
        ));
    }
}
