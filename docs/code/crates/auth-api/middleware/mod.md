# middleware/mod.rs

## File Metadata

**File Path**: `crates/auth-api/src/middleware/mod.rs`  
**Crate**: `auth-api`  
**Module**: `middleware`  
**Layer**: Adapter (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator that organizes and re-exports all HTTP middleware components.

---

## Module Structure

### Submodules

```rust
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;
```

### Re-exports

```rust
pub use rate_limit::{RateLimiter, rate_limit_middleware};
pub use request_id::{request_id_middleware, REQUEST_ID_HEADER};
pub use security_headers::security_headers_middleware;
```

---

## Middleware Categories

### 1. Security Middleware
- `security_headers` - CSP, HSTS, X-Frame-Options
- `rate_limit` - Request rate limiting

### 2. Observability Middleware
- `request_id` - Request tracking

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 8  
**Security Level**: LOW
