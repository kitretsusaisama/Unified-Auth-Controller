//! Rate Limiting Service for OTP requests

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RateLimitRule {
    pub max_requests: u32,
    pub window_minutes: i64,
}

#[derive(Debug)]
pub struct RateLimiter {
    rules: HashMap<String, RateLimitRule>,
    // In-memory tracking (use Redis in production)
    request_counts: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        
        // Define rate limit rules
        rules.insert(
            "otp_request_per_identifier".to_string(),
            RateLimitRule {
                max_requests: 5,
                window_minutes: 15,
            },
        );
        
        rules.insert(
            "otp_request_per_ip".to_string(),
            RateLimitRule {
                max_requests: 10,
                window_minutes: 15,
            },
        );
        
        rules.insert(
            "otp_verification_per_session".to_string(),
            RateLimitRule {
                max_requests: 5,
                window_minutes: 10,
            },
        );
        
        Self {
            rules,
            request_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if rate limit is exceeded
    pub async fn check_limit(
        &self,
        key: &str,
        rule_name: &str,
    ) -> Result<bool, String> {
        let rule = self.rules.get(rule_name)
            .ok_or_else(|| "Rate limit rule not found".to_string())?;
        
        let mut counts = self.request_counts.write().await;
        let now = Utc::now();
        let window_start = now - Duration::minutes(rule.window_minutes);
        
        // Get or create request history for this key
        let requests = counts.entry(key.to_string())
            .or_insert_with(Vec::new);
        
        // Remove requests outside the window
        requests.retain(|ts| ts > &window_start);
        
        // Check if limit exceeded
        if requests.len() >= rule.max_requests as usize {
            return Ok(false); // Rate limit exceeded
        }
        
        // Record this request
        requests.push(now);
        
        Ok(true) // Within rate limit
    }
    
    /// Get remaining requests for a key
    pub async fn get_remaining(
        &self,
        key: &str,
        rule_name: &str,
    ) -> Result<u32, String> {
        let rule = self.rules.get(rule_name)
            .ok_or_else(|| "Rate limit rule not found".to_string())?;
        
        let counts = self.request_counts.read().await;
        let now = Utc::now();
        let window_start = now - Duration::minutes(rule.window_minutes);
        
        if let Some(requests) = counts.get(key) {
            let recent_count = requests.iter()
                .filter(|ts| ts > &&window_start)
                .count() as u32;
            
            Ok(rule.max_requests.saturating_sub(recent_count))
        } else {
            Ok(rule.max_requests)
        }
    }
    
    /// Get time until reset
    pub async fn get_reset_time(
        &self,
        key: &str,
        rule_name: &str,
    ) -> Result<Option<DateTime<Utc>>, String> {
        let rule = self.rules.get(rule_name)
            .ok_or_else(|| "Rate limit rule not found".to_string())?;
        
        let counts = self.request_counts.read().await;
        
        if let Some(requests) = counts.get(key) {
            if let Some(first) = requests.first() {
                let reset_time = *first + Duration::minutes(rule.window_minutes);
                return Ok(Some(reset_time));
            }
        }
        
        Ok(None)
    }
    
    /// Clear rate limit for a key (admin function)
    pub async fn clear_limit(&self, key: &str) {
        let mut counts = self.request_counts.write().await;
        counts.remove(key);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating rate limit keys
pub fn identifier_key(tenant_id: &Uuid, identifier: &str) -> String {
    format!("otp:identifier:{}:{}", tenant_id, identifier)
}

pub fn ip_key(ip: &str) -> String {
    format!("otp:ip:{}", ip)
}

pub fn session_key(session_id: &Uuid) -> String {
    format!("otp:session:{}", session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new();
        let key = "test:key";
        
        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(limiter.check_limit(key, "otp_request_per_identifier").await.unwrap());
        }
        
        // 6th request should fail
        assert!(!limiter.check_limit(key, "otp_request_per_identifier").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_get_remaining() {
        let limiter = RateLimiter::new();
        let key = "test:remaining";
        
        let remaining = limiter.get_remaining(key, "otp_request_per_identifier").await.unwrap();
        assert_eq!(remaining, 5);
        
        limiter.check_limit(key, "otp_request_per_identifier").await.unwrap();
        
        let remaining = limiter.get_remaining(key, "otp_request_per_identifier").await.unwrap();
        assert_eq!(remaining, 4);
    }
    
    #[tokio::test]
    async fn test_clear_limit() {
        let limiter = RateLimiter::new();
        let key = "test:clear";
        
        // Use up the limit
        for _ in 0..5 {
            limiter.check_limit(key, "otp_request_per_identifier").await.unwrap();
        }
        
        // Should be rate limited
        assert!(!limiter.check_limit(key, "otp_request_per_identifier").await.unwrap());
        
        // Clear the limit
        limiter.clear_limit(key).await;
        
        // Should work again
        assert!(limiter.check_limit(key, "otp_request_per_identifier").await.unwrap());
    }
}
