//! Admin UI module - Server-rendered dashboard with Askama templates
//!
//! This module provides an optional admin interface for the SSO platform.
//! Enable with the `admin-ui` feature flag.

#[cfg(feature = "admin-ui")]
pub mod handlers;

#[cfg(feature = "admin-ui")]
pub mod routes;

#[cfg(feature = "admin-ui")]
pub use routes::admin_router;
