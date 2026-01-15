# repositories/webauthn_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/webauthn_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::webauthn_repository`  
**Layer**: Infrastructure (Persistence)  
**Security-Critical**: ✅ **YES** - Passkey storage

## Purpose

Manages persistence of WebAuthn passkeys for passwordless authentication.

### Problem It Solves

- Passkey storage
- Credential management
- WebAuthn data persistence

---

## Detailed Code Breakdown

### Struct: `PasskeyRecord`

**Purpose**: Database representation of passkey

**Fields**:
- `id`: Credential ID (Base64)
- `user_id`: Owner
- `passkey_json`: Serialized passkey
- `created_at`: Registration time

---

### Struct: `WebauthnRepository`

**Purpose**: Passkey persistence

**Fields**:
- `pool`: MySQL connection pool

---

### Method: `save_passkey()`

**Implementation**: Stores passkey in database

**SQL**:
```sql
INSERT INTO passkeys (id, user_id, passkey_json, created_at)
VALUES (?, ?, ?, NOW())
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database access |
| `serde` | Serialization |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 48  
**Security Level**: CRITICAL
