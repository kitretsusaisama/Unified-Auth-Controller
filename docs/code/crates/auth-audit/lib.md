# lib.rs

## File Metadata

**File Path**: `crates/auth-audit/src/lib.rs`  
**Crate**: `auth-audit`  
**Module**: Root  
**Layer**: Infrastructure (Audit)  
**Security-Critical**: ✅ **YES** - Audit logging

## Purpose

Module aggregator for audit logging components.

---

## Module Structure

```rust
pub mod service;
```

---

## Re-exports

```rust
pub use service::{AuditLog, AuditService};
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 3  
**Security Level**: CRITICAL
