# router.rs

## File Metadata

**File Path**: `crates/auth-api/src/router.rs`  
**Crate**: `auth-api`  
**Module**: `router`  
**Layer**: Adapter (HTTP Routing)  
**Security-Critical**: ⚠️ **MEDIUM** - Route configuration and middleware

## Purpose

Configures HTTP routes and middleware layers for the authentication API, organizing all endpoints and applying cross-cutting concerns.

### Problem It Solves

- Route organization
- Middleware composition
- Request tracing
- Security headers
- Rate limiting
- Request ID tracking

---

## Detailed Code Breakdown

### Function: `api_router()`

**Signature**: `pub fn api_router() -> Router<AppState>`

**Purpose**: Create complete API router with all routes and middleware

**Routes**:

#### Health & Monitoring
```rust
.route("/health", get(health::health_check))
```

#### Authentication
```rust
.route("/auth/login", post(auth::login))
.route("/auth/register", post(auth::register))
```

#### OIDC
```rust
.route("/auth/oidc/login", get(auth_oidc::login))
.route("/auth/oidc/callback", get(auth_oidc::callback))
```

#### SAML
```rust
.route("/auth/saml/metadata", get(auth_saml::metadata))
.route("/auth/saml/acs", post(auth_saml::acs))
```

#### User Management
```rust
.route("/users/:id/ban", post(users::ban_user))
.route("/users/:id/activate", post(users::activate_user))
```

---

## Middleware Stack

**Execution Order**: Bottom to top

```rust
Router::new()
    .route(...)
    // 4. HTTP tracing (outermost)
    .layer(TraceLayer::new_for_http())
    // 3. Security headers
    .layer(middleware::from_fn(security_headers_middleware))
    // 2. Request ID
    .layer(middleware::from_fn(request_id_middleware))
    // 1. Rate limiting (innermost)
    .layer(axum::Extension(rate_limiter))
```

### Middleware Layers

1. **Rate Limiter** (5 req/min)
2. **Request ID** - Generates unique ID
3. **Security Headers** - CSP, HSTS, etc.
4. **Trace Layer** - HTTP request logging

---

## Complete Router Configuration

```rust
use axum::{routing::{get, post}, Router, middleware};
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};
use std::time::Duration;

pub fn api_router() -> Router<AppState> {
    // Create rate limiter
    let rate_limiter = RateLimiter::new(5, Duration::from_secs(60));
    
    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .max_age(Duration::from_secs(3600));
    
    Router::new()
        // Health
        .route("/health", get(health::health_check))
        .route("/metrics", get(metrics::prometheus_metrics))
        
        // Authentication
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/refresh", post(auth::refresh_token))
        
        // OIDC
        .route("/auth/oidc/login", get(auth_oidc::login))
        .route("/auth/oidc/callback", get(auth_oidc::callback))
        
        // SAML
        .route("/auth/saml/metadata", get(auth_saml::metadata))
        .route("/auth/saml/acs", post(auth_saml::acs))
        .route("/auth/saml/login", get(auth_saml::login))
        
        // User Management (admin only)
        .route("/users", get(users::list_users))
        .route("/users/:id", get(users::get_user))
        .route("/users/:id", put(users::update_user))
        .route("/users/:id", delete(users::delete_user))
        .route("/users/:id/ban", post(users::ban_user))
        .route("/users/:id/activate", post(users::activate_user))
        
        // Session Management
        .route("/sessions", get(sessions::list_sessions))
        .route("/sessions/:id", delete(sessions::revoke_session))
        
        // Middleware layers
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(axum::Extension(rate_limiter))
}
```

---

## Route Groups

### Public Routes (No Auth)

```rust
let public_routes = Router::new()
    .route("/health", get(health::health_check))
    .route("/auth/login", post(auth::login))
    .route("/auth/register", post(auth::register))
    .route("/auth/oidc/login", get(auth_oidc::login))
    .route("/auth/oidc/callback", get(auth_oidc::callback))
    .route("/auth/saml/metadata", get(auth_saml::metadata))
    .route("/auth/saml/acs", post(auth_saml::acs));
```

### Protected Routes (Auth Required)

```rust
let protected_routes = Router::new()
    .route("/users", get(users::list_users))
    .route("/sessions", get(sessions::list_sessions))
    .layer(middleware::from_fn(auth_middleware));
```

### Admin Routes (Admin Only)

```rust
let admin_routes = Router::new()
    .route("/users/:id/ban", post(users::ban_user))
    .route("/users/:id/activate", post(users::activate_user))
    .layer(middleware::from_fn(require_admin));
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `tower-http` | HTTP middleware |

### Internal Dependencies

- [handlers/*](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Route handlers
- [middleware/*](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/middleware/security_headers.md) - Middleware

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup
- [server.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/server.rs) - Server

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 28  
**Security Level**: MEDIUM
