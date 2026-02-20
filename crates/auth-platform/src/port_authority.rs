//! Port authority - central coordinator for port management
//!
//! Provides the main API for acquiring and releasing ports with:
//! - Policy enforcement (security classification)
//! - Multi-process coordination (file-based leasing)
//! - OS-level safety (socket reuse options)
//! - Observability (structured logging)

use crate::port_lease::{default_lease_dir, PortLease};
use crate::port_policy::{PortClass, PortPolicy};
use crate::safe_socket::{bind_with_reuse, ManagedListener};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Port authority - manages port lifecycle
pub struct PortAuthority {
    /// In-memory registry of active leases
    lease_registry: Arc<DashMap<u16, PortLease>>,

    /// Directory for lease files
    lease_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("Port {port} is occupied by {service} (PID {pid})")]
    PortOccupied {
        port: u16,
        service: String,
        pid: u32,
    },

    #[error("Admin service '{service}' cannot bind to port {port} (no fallback allowed)")]
    AdminPortUnavailable { service: String, port: u16 },

    #[error("No available ports for service '{service}' in range {start}-{end}")]
    NoPortsAvailable {
        service: String,
        start: u16,
        end: u16,
    },

    #[error("Policy validation failed: {0}")]
    PolicyValidation(#[from] crate::port_policy::PolicyError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Lease error: {0}")]
    Lease(String),
}

impl PortAuthority {
    /// Create a new port authority with default lease directory
    pub fn new() -> std::io::Result<Self> {
        Self::with_lease_dir(default_lease_dir())
    }

    /// Create a new port authority with custom lease directory
    pub fn with_lease_dir(lease_dir: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&lease_dir)?;

        info!(lease_dir = ?lease_dir, "Port authority initialized");

        Ok(Self {
            lease_registry: Arc::new(DashMap::new()),
            lease_dir,
        })
    }

    /// Acquire a port with policy enforcement
    ///
    /// This is the main entry point for securing a port. It will:
    /// 1. Validate the policy
    /// 2. Try preferred port first
    /// 3. Fall back to range if allowed by policy
    /// 4. Bind with OS-level safety options
    /// 5. Create a lease file
    /// 6. Log the acquisition with full context
    pub async fn acquire(
        &self,
        policy: &PortPolicy,
        host: &str,
    ) -> Result<ManagedListener, PortError> {
        // Validate policy first (fail fast)
        policy.validate()?;

        let attempt_number = 1;
        let pid = std::process::id();

        debug!(
            service = %policy.service_name,
            preferred_port = policy.preferred_port,
            class = ?policy.class,
            "Acquiring port"
        );

        // Get candidate ports based on policy
        let candidates = self.get_candidate_ports(policy);

        // Try each candidate port
        for (index, port) in candidates.iter().enumerate() {
            match self.try_acquire_port(*port, host, policy, index > 0).await {
                Ok(listener) => {
                    info!(
                        event = "port.bound",
                        port = port,
                        pid = pid,
                        service = %policy.service_name,
                        class = ?policy.class,
                        attempt = attempt_number,
                        fallback = index > 0,
                        "Port successfully acquired"
                    );

                    return Ok(listener);
                }
                Err(e) => {
                    debug!(
                        port = port,
                        error = %e,
                        "Port not available, trying next"
                    );
                    continue;
                }
            }
        }

        // No ports available - fail based on policy class
        match policy.class {
            PortClass::Admin => Err(PortError::AdminPortUnavailable {
                service: policy.service_name.clone(),
                port: policy.preferred_port,
            }),
            _ => {
                let (start, end) = if let Some(range) = &policy.fallback_range {
                    (*range.start(), *range.end())
                } else {
                    (policy.preferred_port, policy.preferred_port)
                };

                Err(PortError::NoPortsAvailable {
                    service: policy.service_name.clone(),
                    start,
                    end,
                })
            }
        }
    }

    /// Try to acquire a specific port
    async fn try_acquire_port(
        &self,
        port: u16,
        host: &str,
        policy: &PortPolicy,
        is_fallback: bool,
    ) -> Result<ManagedListener, PortError> {
        // Check if port is available (reclaim zombie leases)
        if !PortLease::is_port_available(&self.lease_dir, port).await? {
            // Port has a valid lease
            if let Some(existing_lease) = PortLease::load(&self.lease_dir, port).await? {
                return Err(PortError::PortOccupied {
                    port,
                    service: existing_lease.service_name.clone(),
                    pid: existing_lease.pid,
                });
            }
        }

        // Log warning if falling back
        if is_fallback {
            match policy.class {
                PortClass::Admin => {
                    // Should never happen due to policy validation
                    error!("Admin service attempting fallback - policy violation");
                }
                PortClass::Internal => {
                    warn!(
                        service = %policy.service_name,
                        preferred = policy.preferred_port,
                        actual = port,
                        "Internal service falling back to alternate port"
                    );
                }
                PortClass::Public => {
                    debug!(
                        service = %policy.service_name,
                        preferred = policy.preferred_port,
                        actual = port,
                        "Public service using fallback port"
                    );
                }
            }
        }

        // Try to bind with OS-level safety
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| PortError::Lease(format!("Invalid address: {}", e)))?;

        let listener = bind_with_reuse(addr, &policy.service_name)?;

        // Get the actual bound port (important for port 0 = OS-assigned)
        let actual_port = listener.port();

        // Create and save lease with actual port
        let lease = PortLease::new(actual_port, &policy.service_name);
        lease.save(&self.lease_dir)?;

        // Register in memory with actual port
        self.lease_registry.insert(actual_port, lease);

        Ok(listener)
    }

    /// Get candidate ports based on policy (with process-hash sharding)
    fn get_candidate_ports(&self, policy: &PortPolicy) -> Vec<u16> {
        let mut candidates = vec![policy.preferred_port];

        if let Some(range) = &policy.fallback_range {
            // Use process-hash sharding to spread contention
            let pid = std::process::id();
            let range_vec: Vec<u16> = range.clone().collect();
            let offset = (pid as usize) % range_vec.len();

            // Start from process-specific offset, then try rest
            let mut fallback_ports: Vec<u16> = range_vec
                .iter()
                .cycle()
                .skip(offset)
                .take(range_vec.len())
                .copied()
                .collect();

            candidates.append(&mut fallback_ports);
        }

        candidates
    }

    /// Release a port and cleanup lease
    pub async fn release(&self, port: u16) -> Result<(), PortError> {
        // Remove from in-memory registry
        if let Some((_, lease)) = self.lease_registry.remove(&port) {
            info!(
                event = "port.released",
                port = port,
                service = %lease.service_name,
                "Port released"
            );
        }

        // Delete lease file
        PortLease::delete(&self.lease_dir, port).await?;

        Ok(())
    }

    /// Validate all leases and reclaim zombie locks
    pub async fn validate_leases(&self) -> Result<Vec<u16>, PortError> {
        let mut reclaimed = Vec::new();

        // Read all lease files
        let entries = std::fs::read_dir(&self.lease_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("lease") {
                // Extract port from filename
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Some(port_str) = filename.strip_prefix("port-") {
                        if let Ok(port) = port_str.parse::<u16>() {
                            if PortLease::reclaim(&self.lease_dir, port).await? {
                                reclaimed.push(port);
                            }
                        }
                    }
                }
            }
        }

        if !reclaimed.is_empty() {
            info!(
                reclaimed_ports = ?reclaimed,
                "Zombie leases reclaimed"
            );
        }

        Ok(reclaimed)
    }

    /// Get all active leases
    pub fn active_leases(&self) -> Vec<PortLease> {
        self.lease_registry
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_acquire_preferred_port() {
        let temp_dir = TempDir::new().unwrap();
        let authority = PortAuthority::with_lease_dir(temp_dir.path().to_path_buf()).unwrap();

        let policy = PortPolicy::new(0, PortClass::Public, "test"); // Port 0 = OS assigns
        let listener = authority.acquire(&policy, "127.0.0.1").await.unwrap();

        assert!(listener.port() > 0);
        assert_eq!(listener.service_name(), "test");
    }

    #[tokio::test]
    async fn test_fallback_when_preferred_occupied() {
        let temp_dir = TempDir::new().unwrap();
        let authority = PortAuthority::with_lease_dir(temp_dir.path().to_path_buf()).unwrap();

        // Bind to an ephemeral port first
        let first_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let occupied_port = first_listener.local_addr().unwrap().port();

        // Try to acquire with fallback range
        let policy = PortPolicy::new(occupied_port, PortClass::Public, "test")
            .with_fallback_range((occupied_port + 1)..=(occupied_port + 10));

        let listener = authority.acquire(&policy, "127.0.0.1").await.unwrap();

        // Should get a fallback port
        assert_ne!(listener.port(), occupied_port);
        assert!(listener.port() > occupied_port);
    }

    #[tokio::test]
    async fn test_admin_port_no_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let authority = PortAuthority::with_lease_dir(temp_dir.path().to_path_buf()).unwrap();

        let policy = PortPolicy::new(9000, PortClass::Admin, "admin");

        // Should fail validation if we try to add fallback
        let policy_with_fallback =
            PortPolicy::new(9000, PortClass::Admin, "admin").with_fallback_range(9001..=9010);

        assert!(policy_with_fallback.validate().is_err());

        // Valid admin policy should work
        assert!(policy.validate().is_ok());
    }

    #[tokio::test]
    async fn test_release_port() {
        let temp_dir = TempDir::new().unwrap();
        let authority = PortAuthority::with_lease_dir(temp_dir.path().to_path_buf()).unwrap();

        let policy = PortPolicy::new(0, PortClass::Public, "test");
        let listener = authority.acquire(&policy, "127.0.0.1").await.unwrap();
        let port = listener.port();

        drop(listener); // Close the socket

        // Release the lease
        authority.release(port).await.unwrap();

        // Should be able to acquire again
        let policy2 = PortPolicy::new(port, PortClass::Public, "test2");
        let listener2 = authority.acquire(&policy2, "127.0.0.1").await.unwrap();
        assert_eq!(listener2.port(), port);
    }
}
