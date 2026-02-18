//! Core authentication and authorization logic
//! 
//! This crate contains the pure business logic for the SSO platform,
//! independent of HTTP or database concerns.

pub mod error;
pub mod models;
pub mod services;
pub mod audit;
pub mod resilience;

pub use error::AuthError;

/// Re-export commonly used types
pub mod prelude;