//! OS-level socket safety with Windows compatibility
//!
//! Provides socket binding with proper OS-level options to handle:
//! - Windows TIME_WAIT state preventing immediate port reuse
//! - Unix SO_REUSEPORT for load balancing
//! - Graceful handling of crashed processes

use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, TcpListener};
use tracing::{debug, warn};

/// Managed TCP listener with ownership tracking
pub struct ManagedListener {
    listener: TcpListener,
    port: u16,
    service_name: String,
}

impl ManagedListener {
    /// Create a new managed listener
    pub(crate) fn new(listener: TcpListener, port: u16, service_name: String) -> Self {
        Self {
            listener,
            port,
            service_name,
        }
    }
    
    /// Get the bound port
    pub fn port(&self) -> u16 {
        self.port
    }
    
    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }
    
    /// Get the local address
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.listener.local_addr()
    }
    
    /// Convert into a tokio TcpListener
    pub fn into_tokio_listener(self) -> std::io::Result<tokio::net::TcpListener> {
        self.listener.set_nonblocking(true)?;
        tokio::net::TcpListener::from_std(self.listener)
    }
    
    /// Convert into the underlying std TcpListener
    pub fn into_listener(self) -> TcpListener {
        self.listener
    }
}

/// Bind a socket with OS-level safety options
///
/// This function uses socket2 to set proper socket options before binding:
/// - SO_REUSEADDR: Allows immediate reuse of ports in TIME_WAIT (critical for Windows)
/// - SO_REUSEPORT: On Unix, allows multiple processes to bind to the same port
pub fn bind_with_reuse(addr: SocketAddr, service_name: &str) -> std::io::Result<ManagedListener> {
    let domain = if addr.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };
    
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    
    // CRITICAL: Allows immediate port reuse on Windows after crash/restart
    // Without this, ports remain in TIME_WAIT and refuse to bind
    socket.set_reuse_address(true)?;
    
    #[cfg(unix)]
    {
        // On Unix, also set SO_REUSEPORT for load balancing across processes
        if let Err(e) = socket.set_reuse_port(true) {
            warn!(
                error = %e,
                "Failed to set SO_REUSEPORT (not critical, continuing)"
            );
        }
    }
    
    // Bind to the address
    socket.bind(&addr.into())?;
    
    // Start listening with a reasonable backlog
    socket.listen(1024)?;
    
    debug!(
        addr = %addr,
        service = service_name,
        "Socket bound with reuse options"
    );
    
    let listener: TcpListener = socket.into();
    let port = listener.local_addr()?.port();
    
    Ok(ManagedListener::new(listener, port, service_name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_with_reuse() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap(); // Port 0 = OS assigns
        let listener = bind_with_reuse(addr, "test").unwrap();
        
        assert!(listener.port() > 0);
        assert_eq!(listener.service_name(), "test");
    }

    #[test]
    fn test_immediate_rebind() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        
        // Bind once
        let listener1 = bind_with_reuse(addr, "test1").unwrap();
        let bound_addr = listener1.local_addr().unwrap();
        drop(listener1); // Release the port
        
        // Should be able to rebind immediately
        let listener2 = bind_with_reuse(bound_addr, "test2").unwrap();
        assert_eq!(listener2.local_addr().unwrap(), bound_addr);
    }

    #[tokio::test]
    async fn test_convert_to_tokio() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = bind_with_reuse(addr, "test").unwrap();
        
        let tokio_listener = listener.into_tokio_listener().unwrap();
        assert!(tokio_listener.local_addr().is_ok());
    }
}
