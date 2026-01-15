//! Core configuration structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub features: FeatureConfig,
    pub logging: LoggingConfig,
    pub external_services: ExternalServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    pub host: String,
    pub workers: Option<usize>,
    pub max_connections: Option<u32>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    #[serde(skip_serializing)]
    pub mysql_url: secrecy::Secret<String>,
    pub sqlite_url: Option<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    #[serde(skip_serializing)]
    pub jwt_secret: secrecy::Secret<String>,
    pub jwt_expiry_minutes: u32,
    pub refresh_token_expiry_days: u32,
    pub password_min_length: u8,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
    pub require_mfa: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enabled_features: HashMap<String, bool>,
    pub feature_limits: HashMap<String, u64>,
    pub tenant_overrides: HashMap<String, HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
    pub structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub smtp: Option<SmtpConfig>,
    pub sms: Option<SmsConfig>,
    pub redis: Option<RedisConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: secrecy::Secret<String>,
    pub from_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub provider: String,
    #[serde(skip_serializing)]
    pub api_key: secrecy::Secret<String>,
    pub from_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8081,
                host: "0.0.0.0".to_string(),
                workers: None,
                max_connections: Some(1000),
                timeout_seconds: Some(30),
            },
            database: DatabaseConfig {
                mysql_url: secrecy::Secret::new("mysql://localhost/auth".to_string()),
                sqlite_url: Some(":memory:".to_string()),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 3600,
            },
            security: SecurityConfig {
                jwt_secret: secrecy::Secret::new("change-me-in-production".to_string()),
                jwt_expiry_minutes: 15,
                refresh_token_expiry_days: 30,
                password_min_length: 8,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
                require_mfa: false,
                allowed_origins: vec!["http://localhost:3000".to_string()],
            },
            features: FeatureConfig {
                enabled_features: HashMap::new(),
                feature_limits: HashMap::new(),
                tenant_overrides: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                structured: true,
            },
            external_services: ExternalServicesConfig {
                smtp: None,
                sms: None,
                redis: None,
            },
        }
    }
}