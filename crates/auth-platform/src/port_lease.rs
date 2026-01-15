//! Multi-process port leasing with PID validation
//!
//! Provides file-based port leasing to prevent port conflicts across multiple
//! processes (parallel tests, dev environments, service restarts). Uses PID
//! validation to detect and reclaim zombie locks from crashed processes.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use sysinfo::{Pid, System};
use tracing::{debug, info};

/// Port lease with process ownership tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortLease {
    /// Leased port number
    pub port: u16,
    
    /// Process ID of the owner
    pub pid: u32,
    
    /// Service name that acquired the lease
    pub service_name: String,
    
    /// When the lease was acquired
    pub acquired_at: SystemTime,
    
    /// Boot ID to detect system reboots (optional)
    #[serde(default)]
    pub boot_id: String,
}

impl PortLease {
    /// Create a new port lease for the current process
    pub fn new(port: u16, service_name: impl Into<String>) -> Self {
        Self {
            port,
            pid: std::process::id(),
            service_name: service_name.into(),
            acquired_at: SystemTime::now(),
            boot_id: Self::get_boot_id(),
        }
    }
    
    /// Check if the owning process is still alive
    pub fn is_valid(&self) -> bool {
        let mut system = System::new();
        system.refresh_processes();
        
        let pid = Pid::from_u32(self.pid);
        let exists = system.process(pid).is_some();
        
        if !exists {
            debug!(
                port = self.port,
                pid = self.pid,
                service = %self.service_name,
                "Lease owner process is dead"
            );
        }
        
        exists
    }
    
    /// Get a boot identifier (simple timestamp-based for now)
    fn get_boot_id() -> String {
        // In production, this could read from /proc/sys/kernel/random/boot_id on Linux
        // or use WMI on Windows. For now, use a simple approach.
        format!("{:?}", SystemTime::now())
    }
    
    /// Save lease to file
    pub fn save(&self, lease_dir: &Path) -> std::io::Result<()> {
        fs::create_dir_all(lease_dir)?;
        
        let lease_path = Self::lease_path(lease_dir, self.port);
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&lease_path, json)?;
        
        debug!(
            port = self.port,
            pid = self.pid,
            path = ?lease_path,
            "Lease saved"
        );
        
        Ok(())
    }
    
    /// Load lease from file
    pub fn load(lease_dir: &Path, port: u16) -> std::io::Result<Option<Self>> {
        let lease_path = Self::lease_path(lease_dir, port);
        
        if !lease_path.exists() {
            return Ok(None);
        }
        
        let json = fs::read_to_string(&lease_path)?;
        let lease: Self = serde_json::from_str(&json)?;
        
        Ok(Some(lease))
    }
    
    /// Delete lease file
    pub fn delete(lease_dir: &Path, port: u16) -> std::io::Result<()> {
        let lease_path = Self::lease_path(lease_dir, port);
        
        if lease_path.exists() {
            fs::remove_file(&lease_path)?;
            debug!(port = port, path = ?lease_path, "Lease deleted");
        }
        
        Ok(())
    }
    
    /// Reclaim lease from a dead process
    pub fn reclaim(lease_dir: &Path, port: u16) -> std::io::Result<bool> {
        if let Some(lease) = Self::load(lease_dir, port)? {
            if !lease.is_valid() {
                info!(
                    port = port,
                    previous_pid = lease.pid,
                    previous_service = %lease.service_name,
                    "Reclaiming zombie lease"
                );
                
                Self::delete(lease_dir, port)?;
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Check if port is available (no valid lease exists)
    pub fn is_port_available(lease_dir: &Path, port: u16) -> std::io::Result<bool> {
        // First try to reclaim any zombie leases
        Self::reclaim(lease_dir, port)?;
        
        // Then check if a valid lease exists
        if let Some(lease) = Self::load(lease_dir, port)? {
            if lease.is_valid() {
                debug!(
                    port = port,
                    owner_pid = lease.pid,
                    owner_service = %lease.service_name,
                    "Port already leased"
                );
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get the lease file path for a port
    fn lease_path(lease_dir: &Path, port: u16) -> PathBuf {
        lease_dir.join(format!("port-{}.lease", port))
    }
}

/// Get the default lease directory
pub fn default_lease_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    temp_dir.join("auth-platform").join("port-leases")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_lease_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let lease_dir = temp_dir.path();
        
        let lease = PortLease::new(8081, "test-service");
        lease.save(lease_dir).unwrap();
        
        let loaded = PortLease::load(lease_dir, 8081).unwrap();
        assert!(loaded.is_some());
        
        let loaded = loaded.unwrap();
        assert_eq!(loaded.port, 8081);
        assert_eq!(loaded.service_name, "test-service");
        assert_eq!(loaded.pid, std::process::id());
    }

    #[test]
    fn test_current_process_is_valid() {
        let lease = PortLease::new(8081, "test");
        assert!(lease.is_valid()); // Current process should be alive
    }

    #[test]
    fn test_dead_process_is_invalid() {
        let mut lease = PortLease::new(8081, "test");
        lease.pid = 99999; // Non-existent PID
        
        assert!(!lease.is_valid());
    }

    #[test]
    fn test_zombie_lease_reclamation() {
        let temp_dir = TempDir::new().unwrap();
        let lease_dir = temp_dir.path();
        
        // Create a lease with a dead PID
        let mut zombie_lease = PortLease::new(8081, "zombie");
        zombie_lease.pid = 99999;
        zombie_lease.save(lease_dir).unwrap();
        
        // Reclaim should succeed
        let reclaimed = PortLease::reclaim(lease_dir, 8081).unwrap();
        assert!(reclaimed);
        
        // Lease should be gone
        let loaded = PortLease::load(lease_dir, 8081).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_port_availability() {
        let temp_dir = TempDir::new().unwrap();
        let lease_dir = temp_dir.path();
        
        // Port should be available initially
        assert!(PortLease::is_port_available(lease_dir, 8081).unwrap());
        
        // Lease the port
        let lease = PortLease::new(8081, "test");
        lease.save(lease_dir).unwrap();
        
        // Port should NOT be available
        assert!(!PortLease::is_port_available(lease_dir, 8081).unwrap());
        
        // Delete the lease
        PortLease::delete(lease_dir, 8081).unwrap();
        
        // Port should be available again
        assert!(PortLease::is_port_available(lease_dir, 8081).unwrap());
    }
}
