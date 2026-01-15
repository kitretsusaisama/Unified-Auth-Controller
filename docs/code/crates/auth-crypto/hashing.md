# hashing.rs

## File Metadata

**File Path**: `crates/auth-crypto/src/hashing.rs`  
**Crate**: `auth-crypto`  
**Module**: `hashing`  
**Layer**: Infrastructure (Cryptography)  
**Security-Critical**: ✅ **YES** - Password hashing

## Purpose

Provides secure password hashing using Argon2id, the recommended algorithm for password storage.

### Problem It Solves

- Secure password hashing
- Password verification
- Resistance to GPU/ASIC attacks
- Memory-hard hashing

---

## Detailed Code Breakdown

### Struct: `PasswordHasher`

**Purpose**: Argon2id password hasher

**Fields**:
- `argon2`: Argon2 instance

---

### Method: `hash_password()`

**Signature**: `pub fn hash_password(&self, password: &str) -> Result<String>`

**Purpose**: Hash password using Argon2id

**Process**:
1. Generate random salt
2. Hash password with Argon2id
3. Return PHC string format

**Example**:
```rust
let hasher = PasswordHasher::new();
let hash = hasher.hash_password("MyP@ssw0rd123")?;
// Returns: $argon2id$v=19$m=19456,t=2,p=1$...
```

---

### Method: `verify_password()`

**Signature**: `pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool>`

**Purpose**: Verify password against hash

**Example**:
```rust
if hasher.verify_password(input, &stored_hash)? {
    // Password correct
}
```

---

## Security Considerations

### 1. Argon2id Algorithm

**Why Argon2id?**
- Winner of Password Hashing Competition (2015)
- Memory-hard (resistant to GPU attacks)
- Side-channel resistant
- Configurable parameters

### 2. Salt Generation

**Requirement**: Cryptographically secure random salt

```rust
let salt = SaltString::generate(&mut OsRng);
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `argon2` | Password hashing |
| `rand_core` | Random salt generation |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 40  
**Security Level**: CRITICAL
