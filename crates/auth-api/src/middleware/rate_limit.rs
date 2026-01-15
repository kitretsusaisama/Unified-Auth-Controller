use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    response::IntoResponse,
};
use dashmap::DashMap;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

/// Rate limiter using token bucket algorithm
#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
    max_tokens: u32,
    refill_rate: Duration,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_rate: Duration) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            max_tokens,
            refill_rate,
        }
    }

    /// Check if request is allowed for given key (e.g., IP address)
    pub fn check_rate_limit(&self, key: &str) -> bool {
        let mut bucket = self.buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.max_tokens as f64,
            last_refill: Instant::now(),
        });

        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill);
        
        // Refill tokens based on elapsed time
        let tokens_to_add = (elapsed.as_secs_f64() / self.refill_rate.as_secs_f64()) * self.max_tokens as f64;
        bucket.tokens = (bucket.tokens + tokens_to_add).min(self.max_tokens as f64);
        bucket.last_refill = now;

        // Try to consume one token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Middleware for rate limiting based on IP address
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Get rate limiter from request extensions
    let limiter = req.extensions().get::<RateLimiter>().cloned();
    
    if let Some(limiter) = limiter {
        let ip = addr.ip().to_string();
        
        if !limiter.check_rate_limit(&ip) {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded. Please try again later.",
            )
                .into_response();
        }
    }

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        
        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("127.0.0.1"));
        }
        
        // 6th request should be blocked
        assert!(!limiter.check_rate_limit("127.0.0.1"));
    }

    #[test]
    fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        
        // Different IPs should have separate limits
        assert!(limiter.check_rate_limit("127.0.0.1"));
        assert!(limiter.check_rate_limit("192.168.1.1"));
    }
}
