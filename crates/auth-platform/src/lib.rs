//! Platform-level infrastructure for enterprise authentication system
//!
//! This crate provides cross-cutting platform concerns that are needed across
//! the entire authentication system, including:
//!
//! - **Port Management**: Production-grade port binding with OS-level safety,
//!   multi-process coordination, security classification, and graceful lifecycle
//! - **Future**: Circuit breakers, distributed tracing coordination, etc.

pub mod port_authority;
pub mod port_lease;
pub mod port_policy;
pub mod safe_socket;
pub mod shutdown;

pub use port_authority::PortAuthority;
pub use port_lease::PortLease;
pub use port_policy::{PortClass, PortPolicy};
pub use safe_socket::ManagedListener;
pub use shutdown::{GracefulShutdown, shutdown_signal};

/// Platform-level errors
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Port error: {0}")]
    Port(#[from] port_authority::PortError),
    
    #[error("Policy error: {0}")]
    Policy(#[from] port_policy::PolicyError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Shutdown error: {0}")]
    Shutdown(String),
}

pub type Result<T> = std::result::Result<T, PlatformError>;
