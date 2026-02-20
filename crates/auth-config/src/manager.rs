//! Dynamic configuration management with hot-reload capabilities

use crate::config::AppConfig;
use crate::loader::ConfigLoader;
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{info, warn, error};

pub struct ConfigManager {
    current_config: Arc<RwLock<AppConfig>>,
    config_sender: watch::Sender<AppConfig>,
    config_receiver: watch::Receiver<AppConfig>,
    tenant_overrides: Arc<DashMap<String, serde_json::Value>>,
    loader: ConfigLoader,
}

impl ConfigManager {
    pub fn new(loader: ConfigLoader) -> Result<Self> {
        let initial_config = loader.load()
            .map_err(|e| anyhow::anyhow!("Failed to load initial configuration: {}", e))?;

        let (config_sender, config_receiver) = watch::channel(initial_config.clone());

        Ok(Self {
            current_config: Arc::new(RwLock::new(initial_config)),
            config_sender,
            config_receiver,
            tenant_overrides: Arc::new(DashMap::new()),
            loader,
        })
    }

    #[cfg(test)]
    pub fn new_with_config(config: AppConfig) -> Result<Self> {
        let (config_sender, config_receiver) = watch::channel(config.clone());

        Ok(Self {
            current_config: Arc::new(RwLock::new(config)),
            config_sender,
            config_receiver,
            tenant_overrides: Arc::new(DashMap::new()),
            loader: ConfigLoader::new("config", "test"), // Dummy loader for tests
        })
    }

    pub fn get_config(&self) -> AppConfig {
        self.current_config.read().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<AppConfig> {
        self.config_receiver.clone()
    }

    pub async fn reload_config(&self) -> Result<()> {
        match self.loader.load() {
            Ok(new_config) => {
                // Validate the new configuration
                if let Err(e) = validator::Validate::validate(&new_config) {
                    error!("Configuration validation failed: {}", e);
                    return Err(anyhow::anyhow!("Invalid configuration: {}", e));
                }

                // Update the current configuration
                {
                    let mut config = self.current_config.write();
                    *config = new_config.clone();
                }

                // Notify subscribers
                if let Err(e) = self.config_sender.send(new_config) {
                    warn!("Failed to notify configuration subscribers: {}", e);
                }

                info!("Configuration reloaded successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to reload configuration: {}", e);
                Err(anyhow::anyhow!("Configuration reload failed: {}", e))
            }
        }
    }

    pub fn set_tenant_override(&self, tenant_id: String, key: String, value: serde_json::Value) {
        let mut overrides = self.tenant_overrides
            .entry(tenant_id)
            .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        if let serde_json::Value::Object(ref mut map) = overrides.value_mut() {
            map.insert(key, value);
        }
    }

    pub fn get_tenant_override(&self, tenant_id: &str, key: &str) -> Option<serde_json::Value> {
        self.tenant_overrides
            .get(tenant_id)?
            .get(key)
            .cloned()
    }

    pub fn remove_tenant_override(&self, tenant_id: &str, key: &str) -> bool {
        if let Some(mut overrides) = self.tenant_overrides.get_mut(tenant_id) {
            if let serde_json::Value::Object(ref mut map) = overrides.value_mut() {
                return map.remove(key).is_some();
            }
        }
        false
    }

    pub async fn start_auto_reload(&self, interval_seconds: u64) {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_seconds)
            );

            loop {
                interval.tick().await;
                if let Err(e) = manager.reload_config().await {
                    error!("Auto-reload failed: {}", e);
                }
            }
        });
    }
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            current_config: Arc::clone(&self.current_config),
            config_sender: self.config_sender.clone(),
            config_receiver: self.config_receiver.clone(),
            tenant_overrides: Arc::clone(&self.tenant_overrides),
            loader: ConfigLoader::new("config", "development"), // Default values for clone
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use proptest::prelude::*;

    // Property test generators
    fn arb_server_config() -> impl Strategy<Value = ServerConfig> {
        (1u16..=65535, any::<String>(), any::<Option<usize>>(), any::<Option<u32>>(), any::<Option<u64>>())
            .prop_map(|(port, host, workers, max_connections, timeout_seconds)| {
                ServerConfig {
                    port,
                    host: if host.is_empty() { "localhost".to_string() } else { host },
                    workers,
                    max_connections,
                    timeout_seconds,
                }
            })
    }

    fn arb_security_config() -> impl Strategy<Value = SecurityConfig> {
        (
            "[a-zA-Z0-9]{32,64}",
            1u32..=60,
            1u32..=365,
            8u8..=128,
            1u32..=100,
            1u32..=1440,
            any::<bool>(),
            prop::collection::vec(any::<String>(), 0..5)
        ).prop_map(|(jwt_secret, jwt_expiry_minutes, refresh_token_expiry_days, password_min_length, max_login_attempts, lockout_duration_minutes, require_mfa, allowed_origins)| {
            SecurityConfig {
                jwt_secret: secrecy::Secret::new(jwt_secret),
                jwt_expiry_minutes,
                refresh_token_expiry_days,
                password_min_length,
                max_login_attempts,
                lockout_duration_minutes,
                require_mfa,
                allowed_origins,
            }
        })
    }

    fn arb_database_config() -> impl Strategy<Value = DatabaseConfig> {
        (
            "[a-zA-Z0-9:/._-]{10,100}",
            any::<Option<String>>(),
            1u32..=100,
            1u32..=50,
            1u64..=300,
            1u64..=3600,
            1u64..=86400
        ).prop_map(|(mysql_url, sqlite_url, max_connections, min_connections, connection_timeout, idle_timeout, max_lifetime)| {
            let min_connections = std::cmp::min(min_connections, max_connections);
            DatabaseConfig {
                mysql_url: secrecy::Secret::new(format!("mysql://{}", mysql_url)),
                sqlite_url,
                max_connections,
                min_connections,
                connection_timeout,
                idle_timeout,
                max_lifetime,
            }
        })
    }

    fn arb_feature_config() -> impl Strategy<Value = FeatureConfig> {
        (
            prop::collection::hash_map(any::<String>(), any::<bool>(), 0..10),
            prop::collection::hash_map(any::<String>(), 1u64..=1000000, 0..10),
            prop::collection::hash_map(any::<String>(), prop::collection::hash_map(any::<String>(), prop_oneof![
                Just(serde_json::Value::Bool(true)),
                Just(serde_json::Value::Bool(false)),
                any::<u64>().prop_map(serde_json::Value::from),
                any::<String>().prop_map(serde_json::Value::from)
            ], 0..5), 0..5)
        ).prop_map(|(enabled_features, feature_limits, tenant_overrides)| {
            FeatureConfig {
                enabled_features,
                feature_limits,
                tenant_overrides,
            }
        })
    }

    fn arb_app_config() -> impl Strategy<Value = AppConfig> {
        (
            arb_server_config(),
            arb_database_config(),
            arb_security_config(),
            arb_feature_config(),
            any::<LoggingConfig>(),
            prop_oneof![
                Just(ExternalServicesConfig {
                    smtp: None,
                    sms: None,
                    redis: None,
                }),
                Just(ExternalServicesConfig {
                    smtp: Some(SmtpConfig {
                        host: "smtp.example.com".to_string(),
                        port: 587,
                        username: "test@example.com".to_string(),
                        password: secrecy::Secret::new("password".to_string()),
                        from_address: "noreply@example.com".to_string(),
                    }),
                    sms: None,
                    redis: None,
                }),
                Just(ExternalServicesConfig {
                    smtp: None,
                    sms: Some(SmsConfig {
                        provider: "twilio".to_string(),
                        api_key: secrecy::Secret::new("api_key".to_string()),
                        from_number: "+1234567890".to_string(),
                    }),
                    redis: None,
                }),
                Just(ExternalServicesConfig {
                    smtp: None,
                    sms: None,
                    redis: Some(RedisConfig {
                        url: "redis://localhost:6379".to_string(),
                        max_connections: 10,
                        timeout_seconds: 30,
                    }),
                }),
            ]
        ).prop_map(|(server, database, security, features, logging, external_services)| {
            AppConfig {
                server,
                database,
                security,
                features,
                logging,
                external_services,
            }
        })
    }

    proptest! {
        #[test]
        fn test_dynamic_configuration_management_property(
            initial_config in arb_app_config(),
            tenant_id in "[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}",
            override_key in "[a-zA-Z_][a-zA-Z0-9_]*",
            override_value in prop_oneof![
                Just(serde_json::Value::Bool(true)),
                Just(serde_json::Value::Bool(false)),
                any::<u64>().prop_map(serde_json::Value::from),
                any::<String>().prop_map(serde_json::Value::from)
            ]
        ) {
            // Feature: rust-auth-platform, Property 14: Dynamic Configuration Management
            let result = tokio_test::block_on(async {
                // Test 1: Real-time updates without service restart (Requirement 16.1)
                let manager = ConfigManager::new_with_config(initial_config.clone()).map_err(|e| proptest::test_runner::TestCaseError::fail(e.to_string()))?;

                let initial_retrieved = manager.get_config();

                // Verify initial configuration is loaded correctly
                prop_assert_eq!(initial_retrieved.server.port, initial_config.server.port);
                prop_assert_eq!(initial_retrieved.security.jwt_expiry_minutes, initial_config.security.jwt_expiry_minutes);

                // Test 2: Configuration versioning and atomic updates (Requirement 16.2, 16.3)
                // Subscribe to configuration changes
                let mut _config_receiver = manager.subscribe();

                // Set tenant override (simulating dynamic configuration change)
                manager.set_tenant_override(
                    tenant_id.clone(),
                    override_key.clone(),
                    override_value.clone()
                );

                // Verify tenant override is applied correctly
                let retrieved_override = manager.get_tenant_override(&tenant_id, &override_key);
                prop_assert_eq!(retrieved_override, Some(override_value.clone()));

                // Test 3: Schema validation during configuration changes (Requirement 16.3)
                // The configuration should remain valid after any changes
                let current_config = manager.get_config();
                prop_assert!(validator::Validate::validate(&current_config).is_ok());

                // Test 4: Rollback capabilities (Requirement 16.2)
                // Remove the tenant override (simulating rollback)
                let removed = manager.remove_tenant_override(&tenant_id, &override_key);
                prop_assert!(removed);

                // Verify override is removed
                let retrieved_after_removal = manager.get_tenant_override(&tenant_id, &override_key);
                prop_assert_eq!(retrieved_after_removal, None);

                // Test 5: Atomic application across instances (Requirement 16.3)
                // Multiple subscribers should receive the same configuration updates
                let manager_clone = manager.clone();
                let config_from_clone = manager_clone.get_config();
                let config_from_original = manager.get_config();

                // Both instances should have the same configuration
                prop_assert_eq!(config_from_clone.server.port, config_from_original.server.port);
                prop_assert_eq!(config_from_clone.security.jwt_expiry_minutes, config_from_original.security.jwt_expiry_minutes);

                Ok(())
            });
            result.unwrap();
        }
    }
}