# models.rs

## File Metadata

**File Path**: `crates/auth-core/src/models.rs`  
**Crate**: `auth-core`  
**Module**: `models`  
**Layer**: Domain (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator that organizes and re-exports all domain models, providing a clean public API for the auth-core crate.

---

## Module Structure

### Submodules

```rust
pub mod user;
pub mod tenant;
pub mod organization;
pub mod role;
pub mod permission;
pub mod token;
pub mod session;
pub mod subscription;
pub mod user_tenant;
pub mod password_policy;
```

### Re-exports

```rust
pub use user::*;
pub use tenant::*;
pub use organization::*;
pub use role::*;
pub use permission::*;
pub use token::*;
pub use session::*;
pub use user_tenant::*;
pub use password_policy::*;
```

---

## Usage

### Import Pattern

```rust
// Instead of:
use auth_core::models::user::User;
use auth_core::models::role::Role;
use auth_core::models::session::Session;

// Use:
use auth_core::models::{User, Role, Session};
```

---

## Model Categories

### 1. Identity Models
- `User` - User accounts
- `Session` - Active sessions
- `Token` - JWT tokens

### 2. Authorization Models
- `Role` - RBAC roles
- `Permission` - Fine-grained permissions

### 3. Multi-Tenancy Models
- `Tenant` - Tenant configuration
- `Organization` - Organization hierarchy
- `UserTenant` - User-tenant relationships

### 4. Subscription Models
- `SubscriptionPlan` - Available plans
- `TenantSubscription` - Active subscriptions

### 5. Security Models
- `PasswordPolicy` - Password rules

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/lib.md) - Crate root

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 22  
**Security Level**: LOW
