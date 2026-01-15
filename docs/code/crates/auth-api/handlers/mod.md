# handlers/mod.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/mod.rs`  
**Crate**: `auth-api`  
**Module**: `handlers`  
**Layer**: Adapter (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator that organizes and re-exports all HTTP request handlers.

---

## Module Structure

### Submodules

```rust
pub mod auth;
pub mod users;
pub mod health;
pub mod auth_oidc;
pub mod auth_saml;
```

---

## Handler Categories

### 1. Authentication Handlers
- `auth` - Standard login/register
- `auth_oidc` - OpenID Connect
- `auth_saml` - SAML 2.0

### 2. User Management
- `users` - User CRUD operations

### 3. Health & Monitoring
- `health` - Health checks

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 7  
**Security Level**: LOW
