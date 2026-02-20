//! Configuration loading from various sources

use crate::config::AppConfig;
use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use std::path::Path;

pub struct ConfigLoader {
    config_dir: String,
    environment: String,
}

impl ConfigLoader {
    pub fn new(config_dir: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            config_dir: config_dir.into(),
            environment: environment.into(),
        }
    }

    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        let mut config = Config::builder();

        // Load default configuration
        config = config.add_source(File::with_name(&format!(
            "{}/default",
            self.config_dir
        )).required(false));

        // Load environment-specific configuration
        config = config.add_source(File::with_name(&format!(
            "{}/{}",
            self.config_dir, self.environment
        )).required(false));

        // Load local configuration (for development)
        config = config.add_source(File::with_name(&format!(
            "{}/local",
            self.config_dir
        )).required(false));

        // Override with environment variables
        config = config.add_source(
            Environment::with_prefix("AUTH")
                .separator("__")
                .try_parsing(true)
        );

        let config = config.build()?;
        config.try_deserialize()
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig, ConfigError> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()?;

        config.try_deserialize()
    }

    pub fn load_from_env() -> Result<AppConfig, ConfigError> {
        let config = Config::builder()
            .add_source(
                Environment::with_prefix("AUTH")
                    .separator("__")
                    .try_parsing(true)
            )
            .build()?;

        config.try_deserialize()
    }
}