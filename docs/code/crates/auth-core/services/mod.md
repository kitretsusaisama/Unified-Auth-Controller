# services/mod.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/mod.rs`  
**Crate**: `auth-core`  
**Module**: `services`  
**Layer**: Domain (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator that organizes and re-exports all business logic services, providing a clean public API for auth-core services.

---

## Module Structure

### Submodules

```rust
pub mod authorization;
pub mod credential;
pub mod identity;
pub mod risk_assessment;
pub mod role_service;
pub mod session_service;
pub mod subscription_service;
pub mod token_service;
pub mod webauthn_service;
```

### Re-exports

```rust
pub use authorization::*;
pub use credential::*;
pub use identity::*;
pub use risk_assessment::*;
pub use role_service::*;
pub use session_service::*;
pub use subscription_service::*;
pub use token_service::*;
pub use webauthn_service::*;
```

---

## Service Categories

### 1. Authentication Services
- `IdentityService` - User authentication
- `CredentialService` - Password management
- `TokenEngine` - JWT token management

### 2. Authorization Services
- `AuthorizationProvider` - RBAC/ABAC
- `RoleService` - Role management

### 3. Session Services
- `SessionService` - Session lifecycle
- `RiskAssessor` - Risk assessment

### 4. Subscription Services
- `SubscriptionService` - Plan management

### 5. Advanced Authentication
- `WebAuthnService` - Passwordless auth

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/lib.md) - Crate root

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 20  
**Security Level**: LOW
