# repositories/mod.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/mod.rs`  
**Crate**: `auth-db`  
**Module**: `repositories`  
**Layer**: Infrastructure (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator for all repository implementations.

---

## Module Structure

```rust
pub mod refresh_token_repository;
pub mod revoked_token_repository;
pub mod role_repository;
pub mod session_repository;
pub mod subscription_repository;
pub mod user_repository;
pub mod webauthn_repository;
```

---

## Re-exports

```rust
pub use refresh_token_repository::*;
pub use revoked_token_repository::*;
pub use role_repository::*;
pub use user_repository::*;
pub use webauthn_repository::*;
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 17  
**Security Level**: LOW
