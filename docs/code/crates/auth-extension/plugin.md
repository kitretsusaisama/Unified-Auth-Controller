# plugin.rs

## File Metadata

**File Path**: `crates/auth-extension/src/plugin.rs`  
**Crate**: `auth-extension`  
**Module**: `plugin`  
**Layer**: Extension (Scripting)  
**Security-Critical**: ⚠️ **MEDIUM** - Plugin execution

## Purpose

Provides plugin system using Rhai scripting engine for extensible authentication logic.

### Problem It Solves

- Custom authentication logic
- Hook-based extensions
- Script-based customization
- Dynamic behavior modification

---

## Detailed Code Breakdown

### Struct: `PluginEngine`

**Purpose**: Rhai script execution engine

**Fields**:
- `engine`: Rhai engine
- `scripts`: Registered scripts (AST)

---

### Method: `register_script()`

**Signature**: `pub async fn register_script(&self, script: &str) -> Result<(), Box<EvalAltResult>>`

**Purpose**: Register plugin script

**Example**:
```rust
let engine = PluginEngine::new();
engine.register_script(r#"
    fn on_login(payload_json) {
        // Custom logic
        payload_json
    }
"#).await?;
```

---

### Method: `execute_hook()`

**Signature**: `pub async fn execute_hook(&self, hook_name: &str, payload: Value) -> Result<Value, Box<EvalAltResult>>`

**Purpose**: Execute hook function

**Example**:
```rust
let result = engine.execute_hook("on_login", json!({
    "user_id": "123",
    "email": "user@example.com"
})).await?;
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 76  
**Security Level**: MEDIUM
