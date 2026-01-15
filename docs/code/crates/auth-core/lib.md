# lib.rs (auth-core)

## File Metadata

**File Path**: `crates/auth-core/src/lib.rs`  
**Crate**: `auth-core`  
**Module**: Root  
**Layer**: Domain + Application  
**Security-Critical**: ⚠️ **MEDIUM** - Entry point for business logic

## Purpose

Root module for the `auth-core` crate, defining the public API and module structure for core authentication and authorization logic.

### Problem It Solves

- Provides clean public API for business logic
- Organizes domain models and application services
- Enables dependency-free business logic (no HTTP or database concerns)
- Facilitates testing and reusability

---

## Detailed Code Breakdown

### Module Structure

```rust
pub mod error;    // Error types
pub mod models;   // Domain models
pub mod services; // Application services
```

**Architecture**:
- **Domain Layer**: `models/` - Pure business entities
- **Application Layer**: `services/` - Use case orchestration
- **Error Handling**: `error.rs` - Centralized error types

---

### Public Re-exports

#### Direct Export
```rust
pub use error::AuthError;
```

**Purpose**: Convenience import for most common error type

**Usage**:
```rust
use auth_core::AuthError;  // Instead of auth_core::error::AuthError
```

---

#### Prelude Module
```rust
pub mod prelude {
    pub use crate::error::AuthError;
    pub use crate::models::*;
    pub use crate::services::*;
}
```

**Purpose**: Glob import for all commonly used types

**Usage**:
```rust
use auth_core::prelude::*;

// Now available:
// - AuthError
// - User, CreateUserRequest, UserStatus
// - Token, RefreshToken, Claims
// - IdentityService, AuthorizationService
// - etc.
```

**Pattern**: Rust prelude pattern for ergonomic imports

---

## Module Organization

### error Module

**File**: [error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/error.rs)

**Exports**:
- `AuthError` - Main error enum
- `TokenErrorKind` - Token-specific errors

**Usage**: Error handling across all auth operations

---

### models Module

**File**: `models/mod.rs`

**Submodules**:
- [user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [token.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/token.rs) - Token models
- [session.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/session.rs) - Session entity
- [role.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/role.rs) - RBAC roles
- `permission.rs` - Permission model
- `tenant.rs` - Tenant model
- `organization.rs` - Organization hierarchy
- `password_policy.rs` - Password rules
- `subscription.rs` - Subscription tiers
- `user_tenant.rs` - User-tenant mapping

**Purpose**: Domain entities and value objects

---

### services Module

**File**: `services/mod.rs`

**Submodules**:
- [identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - Authentication
- `authorization.rs` - Authorization logic
- `credential.rs` - Credential management
- `token_service.rs` - Token operations
- `session_service.rs` - Session management
- `role_service.rs` - Role management
- `risk_assessment.rs` - Risk evaluation
- `subscription_service.rs` - Subscription logic

**Purpose**: Application services (use case orchestration)

---

## Dependency Rules

### Zero External Dependencies

**auth-core** has minimal dependencies:
- `auth-config` - Configuration types
- `auth-crypto` - Cryptographic operations

**No Dependencies On**:
- HTTP frameworks (Axum, etc.)
- Database libraries (SQLx, etc.)
- External APIs

**Benefit**: Pure business logic, easily testable

---

### Dependency Injection

**Pattern**: Services depend on traits, not implementations

```rust
// Service depends on trait
pub struct IdentityService {
    store: Arc<dyn UserStore>,  // Trait, not concrete type
    token_service: Arc<dyn TokenProvider>,
}

// Trait defined in auth-core
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) 
        -> Result<Option<User>, AuthError>;
    // ...
}

// Implementation in auth-db
impl UserStore for UserRepository {
    // Concrete implementation
}
```

**Benefits**:
- Testability (mock implementations)
- Flexibility (swap implementations)
- Separation of concerns

---

## Usage Examples

### Basic Import

```rust
use auth_core::AuthError;
use auth_core::models::User;
use auth_core::services::identity::IdentityService;
```

### Prelude Import

```rust
use auth_core::prelude::*;

// All types available
let user = User { ... };
let error = AuthError::InvalidCredentials;
```

### Service Composition

```rust
use auth_core::services::identity::IdentityService;
use auth_db::repositories::user_repository::UserRepository;

let user_repo = Arc::new(UserRepository::new(pool));
let token_service = Arc::new(TokenEngine::new().await?);
let identity_service = Arc::new(IdentityService::new(user_repo, token_service));
```

---

## Testing

### Unit Tests

**Location**: Within each module file

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_can_authenticate() {
        let user = User {
            status: UserStatus::Active,
            locked_until: None,
            // ...
        };
        assert!(user.can_authenticate());
    }
}
```

### Integration Tests

**Location**: `tests/` directory

```rust
// tests/integration_test.rs
use auth_core::prelude::*;

#[tokio::test]
async fn test_full_authentication_flow() {
    // Test complete flow
}
```

---

## Documentation

### Crate-Level Documentation

```rust
//! Core authentication and authorization logic
//! 
//! This crate contains the pure business logic for the SSO platform,
//! independent of HTTP or database concerns.
```

**Generated**: `cargo doc` produces HTML documentation

**Accessible**: `target/doc/auth_core/index.html`

---

## Architecture Benefits

### 1. Separation of Concerns

**Domain Logic**: Isolated from infrastructure  
**No HTTP**: Business logic doesn't know about HTTP  
**No Database**: Business logic doesn't know about SQL

### 2. Testability

**Mock Implementations**: Easy to create for testing  
**No External Dependencies**: Tests run fast  
**Pure Functions**: Deterministic, easy to test

### 3. Reusability

**Multiple Adapters**: Can be used by HTTP API, CLI, gRPC, etc.  
**Language Agnostic**: Business rules independent of framework

### 4. Maintainability

**Clear Boundaries**: Easy to understand what belongs where  
**Single Responsibility**: Each module has one job  
**Dependency Direction**: Always inward (toward domain)

---

## Related Files

- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - Authentication service
- [error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/error.rs) - Error types
- [Cargo.toml](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/Cargo.toml) - Crate configuration

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 17  
**Security Level**: MEDIUM
