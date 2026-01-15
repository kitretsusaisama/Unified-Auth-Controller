# lib.rs

## File Metadata

**File Path**: `crates/auth-crypto/src/lib.rs`  
**Crate**: `auth-crypto`  
**Module**: Root  
**Layer**: Infrastructure (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator for cryptographic operations.

---

## Module Structure

```rust
pub mod kms;
```

---

## Re-exports

```rust
pub use kms::{KeyProvider, SoftKeyProvider, HsmKeyProvider};
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 3  
**Security Level**: LOW
