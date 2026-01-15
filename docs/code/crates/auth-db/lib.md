# lib.rs

## File Metadata

**File Path**: `crates/auth-db/src/lib.rs`  
**Crate**: `auth-db`  
**Module**: Root  
**Layer**: Infrastructure (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator for database layer components.

---

## Module Structure

```rust
pub mod connection;
pub mod migrations;
pub mod repositories;
pub mod models;
pub mod sharding;
```

---

## Re-exports

```rust
pub use connection::*;
pub use repositories::*;
pub use sharding::*;
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 11  
**Security Level**: LOW
