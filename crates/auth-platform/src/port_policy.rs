//! Port policy and security classification
//!
//! Defines port allocation policies with security-aware classification to prevent
//! accidental exposure of privileged services.

use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

/// Security classification for port binding behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortClass {
    /// Public-facing services (HTTP, etc.) - can fallback to alternative ports
    Public,

    /// Internal services (metrics, health checks) - can fallback with warning
    Internal,

    /// Admin/privileged services - NEVER fallback, fail fast if unavailable
    Admin,
}

/// Port allocation policy with security constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortPolicy {
    /// Preferred port to bind
    pub preferred_port: u16,

    /// Optional range for fallback ports (None = fail if preferred unavailable)
    pub fallback_range: Option<RangeInclusive<u16>>,

    /// Security classification determines fallback behavior
    pub class: PortClass,

    /// Service name for logging and lease tracking
    pub service_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Privileged port {0} not allowed (must be >= 1024)")]
    PrivilegedPort(u16),

    #[error("Admin service '{0}' cannot have fallback range")]
    AdminFallbackNotAllowed(String),

    #[error("Invalid fallback range: {start} to {end}")]
    InvalidRange { start: u16, end: u16 },

    #[error("Port range {start}-{end} overlaps with privileged ports")]
    PrivilegedRangeOverlap { start: u16, end: u16 },
}

impl PortPolicy {
    /// Create a new port policy
    pub fn new(
        preferred_port: u16,
        class: PortClass,
        service_name: impl Into<String>,
    ) -> Self {
        Self {
            preferred_port,
            fallback_range: None,
            class,
            service_name: service_name.into(),
        }
    }

    /// Set fallback range for non-admin services
    pub fn with_fallback_range(mut self, range: RangeInclusive<u16>) -> Self {
        self.fallback_range = Some(range);
        self
    }

    /// Validate port policy before use
    pub fn validate(&self) -> Result<(), PolicyError> {
        // Prevent privileged ports
        if self.preferred_port < 1024 {
            return Err(PolicyError::PrivilegedPort(self.preferred_port));
        }

        // Admin ports cannot have fallback (security-critical)
        if self.class == PortClass::Admin && self.fallback_range.is_some() {
            return Err(PolicyError::AdminFallbackNotAllowed(
                self.service_name.clone(),
            ));
        }

        // Validate fallback range if present
        if let Some(range) = &self.fallback_range {
            if range.start() >= range.end() {
                return Err(PolicyError::InvalidRange {
                    start: *range.start(),
                    end: *range.end(),
                });
            }

            if *range.start() < 1024 {
                return Err(PolicyError::PrivilegedRangeOverlap {
                    start: *range.start(),
                    end: *range.end(),
                });
            }
        }

        Ok(())
    }

    /// Get all candidate ports (preferred + fallback range)
    pub fn candidate_ports(&self) -> Vec<u16> {
        let mut ports = vec![self.preferred_port];

        if let Some(range) = &self.fallback_range {
            ports.extend(range.clone());
        }

        ports
    }
}

impl Default for PortPolicy {
    fn default() -> Self {
        Self {
            preferred_port: 8081,
            fallback_range: Some(8082..=8090),
            class: PortClass::Public,
            service_name: "default".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privileged_port_rejected() {
        let policy = PortPolicy::new(80, PortClass::Public, "http");
        assert!(matches!(
            policy.validate(),
            Err(PolicyError::PrivilegedPort(80))
        ));
    }

    #[test]
    fn test_admin_fallback_rejected() {
        let policy = PortPolicy::new(9000, PortClass::Admin, "admin")
            .with_fallback_range(9001..=9010);

        assert!(matches!(
            policy.validate(),
            Err(PolicyError::AdminFallbackNotAllowed(_))
        ));
    }

    #[test]
    fn test_valid_public_policy() {
        let policy = PortPolicy::new(8081, PortClass::Public, "http")
            .with_fallback_range(8082..=8090);

        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_candidate_ports() {
        let policy = PortPolicy::new(8081, PortClass::Public, "http")
            .with_fallback_range(8082..=8084);

        let candidates = policy.candidate_ports();
        assert_eq!(candidates, vec![8081, 8082, 8083, 8084]);
    }
}
