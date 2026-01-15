//! Resilience utilities for retry logic
//! 
//! Provides standardized retry policies for external operations.

use std::time::Duration;
use rand::Rng;
use std::future::Future;

/// Configuration for retry logic
#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
        }
    }
}

/// Execute a future with exponential backoff retry
pub async fn retry<F, Fut, T, E>(
    config: RetryConfig, 
    mut operation: F
) -> Result<T, E> 
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display, // Requirement for logging, flexible
{
    let mut attempt = 1;
    let mut delay = config.base_delay_ms;
    
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt >= config.max_attempts {
                    return Err(e);
                }
                
                // Calculate delay with jitter
                let jitter = rand::thread_rng().gen_range(0..=config.base_delay_ms / 2);
                let current_delay = delay + jitter;
                
                // Cap at max delay
                let sleep_ms = current_delay.min(config.max_delay_ms);
                
                tracing::warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempt,
                    config.max_attempts,
                    e,
                    sleep_ms
                );
                
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                
                attempt += 1;
                delay = (delay * 2).min(config.max_delay_ms);
            }
        }
    }
}
