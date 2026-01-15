# lib.rs

## File Metadata

**File Path**: `crates/auth-extension/src/lib.rs`  
**Crate**: `auth-extension`  
**Module**: Root  
**Layer**: Extension (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator for extension mechanisms.

---

## Module Structure

```rust
pub mod plugin;
pub mod webhook;
pub mod graphql;
```

---

## Re-exports

```rust
pub use plugin::PluginEngine;
pub use webhook::WebhookDispatcher;
pub use graphql::create_schema;
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 8  
**Security Level**: LOW
