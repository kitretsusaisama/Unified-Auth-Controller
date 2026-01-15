# service.rs

## File Metadata

**File Path**: `crates/auth-audit/src/service.rs`  
**Crate**: `auth-audit`  
**Module**: `service`  
**Layer**: Infrastructure (Audit)  
**Security-Critical**: ✅ **YES** - Audit logging with blockchain-like integrity

## Purpose

Provides immutable audit logging with hash chaining for tamper detection and compliance reporting.

### Problem It Solves

- Immutable audit trail
- Tamper detection
- Compliance reporting (CEF format)
- Audit log integrity verification

---

## Detailed Code Breakdown

### Struct: `AuditLog`

**Purpose**: Audit log entry with hash chaining

**Fields**:
- `id`: Unique identifier
- `action`: Action performed
- `actor_id`: User who performed action
- `resource`: Resource affected
- `metadata`: Additional context (JSON)
- `timestamp`: When action occurred
- `hash`: Hash of current entry
- `prev_hash`: Hash of previous entry (blockchain-like)

---

### Struct: `AuditService`

**Purpose**: Audit logging service

---

### Method: `log()`

**Signature**: `pub async fn log(&self, action: &str, actor_id: Uuid, resource: &str, metadata: Option<Value>) -> Result<AuditLog>`

**Purpose**: Create immutable audit log entry

**Process**:
1. Fetch previous log entry
2. Get previous hash
3. Compute current hash (prev_hash + id + action + actor + resource + timestamp)
4. Store in database
5. Return audit log

**Hash Chaining**:
```
Log 1: hash1 = SHA256(0 + id1 + action1 + ...)
Log 2: hash2 = SHA256(hash1 + id2 + action2 + ...)
Log 3: hash3 = SHA256(hash2 + id3 + action3 + ...)
```

**Example**:
```rust
let audit_service = AuditService::new(pool);

audit_service.log(
    "user.login",
    user_id,
    "session:123",
    Some(json!({
        "ip": "192.168.1.1",
        "user_agent": "Mozilla/5.0"
    }))
).await?;
```

---

### Method: `export_cef()`

**Signature**: `pub fn export_cef(&self, log: &AuditLog) -> String`

**Purpose**: Export audit log in CEF (Common Event Format) for SIEM integration

**Format**: `CEF:Version|Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|[Extension]`

**Example Output**:
```
CEF:0|AuthPlatform|SSO|1.0|user.login|user.login|5|act=user123 msg=session:123
```

---

### Method: `verify_chain()`

**Signature**: `pub async fn verify_chain(&self) -> Result<bool>`

**Purpose**: Verify audit log integrity

**Process**:
1. Fetch all logs in order
2. Recompute each hash
3. Verify hash matches stored hash
4. Verify prev_hash chain

---

## Security Considerations

### 1. Immutability

**Guarantee**: Logs cannot be modified without detection

**Detection**: Hash chain breaks if any log is tampered

### 2. Tamper Detection

**Example**:
```
Original: Log1 → Log2 → Log3
Tampered: Log1 → Log2' → Log3
Result: hash(Log2') ≠ prev_hash(Log3) → TAMPER DETECTED
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database |
| `serde` | Serialization |
| `chrono` | Timestamps |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 115  
**Security Level**: CRITICAL
