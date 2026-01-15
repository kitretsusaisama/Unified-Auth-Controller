//! Configuration validation utilities

use crate::config::AppConfig;
use thiserror::Error;
use validator::{Validate, ValidationErrors};
use secrecy::ExposeSecret;

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
                message: "Max connections must be greater than or equal to min connections".to_string(),
            });
        }
        
        if db.max_connections > 1000 {
            return Err(ConfigValidationError::DatabaseValidationFailed {
                message: "Max connections should not exceed 1000 for performance reasons".to_string(),
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