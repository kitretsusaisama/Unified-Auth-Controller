//! Configuration management system
//! 
//! Provides dynamic configuration management with real-time updates,
//! versioning, and validation capabilities.

pub mod config;
pub mod loader;
pub mod manager;
pub mod validation;

pub use config::*;
pub use loader::*;
pub use manager::*;
pub use validation::*;