# middleware/rate_limit.rs

## File Metadata

**File Path**: `crates/auth-api/src/middleware/rate_limit.rs`  
**Crate**: `auth-api`  
**Module**: `middleware::rate_limit`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - DDoS and brute-force protection

## Purpose

Implements token bucket rate limiting middleware to protect against DDoS attacks, brute-force attempts, and API abuse by limiting requests per IP address.

### Problem It Solves

- Prevents brute-force login attacks
- Protects against DDoS attacks
- Prevents API abuse
- Ensures fair resource usage

---

## Detailed Code Breakdown

### Struct: `RateLimiter`

**Purpose**: Token bucket rate limiter with per-IP tracking

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `buckets` | `Arc<DashMap<String, TokenBucket>>` | Per-IP token buckets |
| `max_tokens` | `u32` | Maximum tokens in bucket |
| `refill_rate` | `Duration` | Time to refill all tokens |

---

### Struct: `TokenBucket`

**Purpose**: Token bucket for single IP

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `tokens` | `f64` | Current token count |
| `last_refill` | `Instant` | Last refill timestamp |

---

### Method: `RateLimiter::new()`

**Signature**: `pub fn new(max_tokens: u32, refill_rate: Duration) -> Self`

**Purpose**: Create rate limiter with configuration

**Parameters**:
- `max_tokens` - Burst capacity (e.g., 100 requests)
- `refill_rate` - Time to fully refill (e.g., 60 seconds)

**Example**:
```rust
// Allow 100 requests per minute
let limiter = RateLimiter::new(100, Duration::from_secs(60));

// Allow 5 requests per minute (stricter)
let limiter = RateLimiter::new(5, Duration::from_secs(60));
```

---

### Method: `check_rate_limit()`

**Signature**: `pub fn check_rate_limit(&self, key: &str) -> bool`

**Purpose**: Check if request is allowed for key (IP address)

**Algorithm**: Token Bucket

**Steps**:

1. **Get or Create Bucket**
   ```rust
   let mut bucket = self.buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
       tokens: self.max_tokens as f64,
       last_refill: Instant::now(),
   });
   ```

2. **Calculate Refill**
   ```rust
   let now = Instant::now();
   let elapsed = now.duration_since(bucket.last_refill);
   
   let tokens_to_add = (elapsed.as_secs_f64() / self.refill_rate.as_secs_f64()) 
                       * self.max_tokens as f64;
   bucket.tokens = (bucket.tokens + tokens_to_add).min(self.max_tokens as f64);
   bucket.last_refill = now;
   ```

3. **Consume Token**
   ```rust
   if bucket.tokens >= 1.0 {
       bucket.tokens -= 1.0;
       true  // Allow request
   } else {
       false  // Block request
   }
   ```

**Returns**: `true` if allowed, `false` if rate limited

---

## Token Bucket Algorithm

### Concept

**Bucket**: Container with tokens  
**Request**: Consumes 1 token  
**Refill**: Tokens added over time

**Formula**:
```
tokens_to_add = (elapsed_time / refill_rate) * max_tokens
current_tokens = min(current_tokens + tokens_to_add, max_tokens)
```

### Example

**Configuration**:
- `max_tokens = 10`
- `refill_rate = 60 seconds`

**Timeline**:
```
t=0s:  tokens=10, request → tokens=9 ✅
t=1s:  tokens=9,  request → tokens=8 ✅
t=2s:  tokens=8,  request → tokens=7 ✅
...
t=10s: tokens=0,  request → BLOCKED ❌
t=30s: tokens=5 (refilled), request → tokens=4 ✅
t=60s: tokens=10 (fully refilled)
```

---

### Middleware Function: `rate_limit_middleware()`

**Signature**:
```rust
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response
```

**Purpose**: Axum middleware for rate limiting

**Process**:

1. **Extract IP Address**
   ```rust
   let ip = addr.ip().to_string();
   ```

2. **Get Rate Limiter**
   ```rust
   let limiter = req.extensions().get::<RateLimiter>().cloned();
   ```

3. **Check Limit**
   ```rust
   if !limiter.check_rate_limit(&ip) {
       return (
           StatusCode::TOO_MANY_REQUESTS,
           "Rate limit exceeded. Please try again later.",
       ).into_response();
   }
   ```

4. **Continue if Allowed**
   ```rust
   next.run(req).await
   ```

---

## Integration

### Axum Router Setup

```rust
use axum::{Router, middleware, Extension};
use std::time::Duration;

let rate_limiter = RateLimiter::new(100, Duration::from_secs(60));

let app = Router::new()
    .route("/api/auth/login", post(login))
    .route("/api/auth/register", post(register))
    .layer(Extension(rate_limiter))
    .layer(middleware::from_fn(rate_limit_middleware));
```

---

## Configuration Patterns

### Pattern 1: Global Rate Limit

```rust
// 1000 requests per hour per IP
let global_limiter = RateLimiter::new(1000, Duration::from_secs(3600));
```

### Pattern 2: Endpoint-Specific Limits

```rust
// Strict limit for login
let login_limiter = RateLimiter::new(5, Duration::from_secs(60));

// Relaxed limit for API
let api_limiter = RateLimiter::new(1000, Duration::from_secs(60));

Router::new()
    .route("/auth/login", post(login))
    .layer(Extension(login_limiter.clone()))
    .layer(middleware::from_fn(rate_limit_middleware))
    .route("/api/users", get(list_users))
    .layer(Extension(api_limiter))
    .layer(middleware::from_fn(rate_limit_middleware));
```

### Pattern 3: User-Based Limiting

```rust
pub async fn user_rate_limit_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Extract user ID from JWT
    let user_id = extract_user_id(&req)?;
    
    let limiter = state.rate_limiter;
    if !limiter.check_rate_limit(&user_id.to_string()) {
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }
    
    next.run(req).await
}
```

---

## Response Headers

### Standard Headers

```rust
if !limiter.check_rate_limit(&ip) {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        "Rate limit exceeded",
    ).into_response();
    
    // Add rate limit headers
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&max_tokens.to_string()).unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str("0").unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        HeaderValue::from_str(&reset_time.to_string()).unwrap(),
    );
    response.headers_mut().insert(
        "Retry-After",
        HeaderValue::from_str("60").unwrap(),
    );
    
    return response;
}
```

---

## Testing

### Unit Tests

```rust
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

#[tokio::test]
async fn test_refill() {
    let limiter = RateLimiter::new(5, Duration::from_millis(100));
    
    // Exhaust tokens
    for _ in 0..5 {
        assert!(limiter.check_rate_limit("127.0.0.1"));
    }
    assert!(!limiter.check_rate_limit("127.0.0.1"));
    
    // Wait for refill
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should allow again
    assert!(limiter.check_rate_limit("127.0.0.1"));
}
```

---

## Production Considerations

### 1. Distributed Rate Limiting

**Problem**: In-memory limiter doesn't work across instances

**Solution**: Use Redis
```rust
use redis::AsyncCommands;

pub async fn check_rate_limit_redis(
    redis: &mut redis::aio::Connection,
    key: &str,
    max_requests: u32,
    window: u64,
) -> Result<bool> {
    let current: u32 = redis.incr(key, 1).await?;
    
    if current == 1 {
        redis.expire(key, window as usize).await?;
    }
    
    Ok(current <= max_requests)
}
```

### 2. Memory Management

**Problem**: Unbounded bucket storage

**Solution**: LRU eviction
```rust
use lru::LruCache;

pub struct RateLimiter {
    buckets: Arc<Mutex<LruCache<String, TokenBucket>>>,
    // ...
}
```

### 3. Bypass for Trusted IPs

```rust
const TRUSTED_IPS: &[&str] = &["10.0.0.1", "192.168.1.1"];

if TRUSTED_IPS.contains(&ip.as_str()) {
    return next.run(req).await;
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `dashmap` | Concurrent hashmap |

---

## Related Files

- [middleware/security_headers.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/middleware/security_headers.md) - Security headers
- [middleware/request_id.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/request_id.rs) - Request tracking

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 110  
**Security Level**: CRITICAL
