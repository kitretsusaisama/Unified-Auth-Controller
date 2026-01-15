# jwt.rs

## File Metadata

**File Path**: `crates/auth-crypto/src/jwt.rs`  
**Crate**: `auth-crypto`  
**Module**: `jwt`  
**Layer**: Infrastructure (Cryptography)  
**Security-Critical**: ✅ **YES** - JWT token operations

## Purpose

Implements JWT token generation and validation using RS256 (RSA with SHA-256) for secure, asymmetric token signing.

### Problem It Solves

- JWT token generation
- Token validation
- RS256 signing/verification
- Claims management
- Token introspection

---

## Detailed Code Breakdown

### Struct: `JwtClaims`

**Purpose**: JWT token claims

**Standard Claims**:
- `sub`: Subject (user ID)
- `iss`: Issuer
- `aud`: Audience
- `exp`: Expiration time
- `iat`: Issued at
- `nbf`: Not before
- `jti`: JWT ID

**Custom Claims**:
- `tenant_id`: Multi-tenancy
- `permissions`: User permissions
- `roles`: User roles
- `scope`: OAuth scope

---

### Struct: `JwtConfig`

**Purpose**: JWT configuration

**Fields**:
- `issuer`: Token issuer
- `audience`: Intended audience
- `access_token_ttl`: Token lifetime (default: 15 minutes)
- `algorithm`: Signing algorithm (RS256)

---

### Struct: `JwtService`

**Purpose**: JWT operations service

**Fields**:
- `config`: JWT configuration
- `key_manager`: Key management

---

### Method: `generate_access_token()`

**Signature**: `pub async fn generate_access_token(&self, user_id: Uuid, tenant_id: Uuid, permissions: Vec<String>, roles: Vec<String>, scope: Option<String>) -> Result<String, JwtError>`

**Purpose**: Generate signed JWT access token

**Process**:
1. Create claims with expiration
2. Get encoding key from KeyManager
3. Sign with RS256
4. Return JWT string

**Example**:
```rust
let token = jwt_service.generate_access_token(
    user_id,
    tenant_id,
    vec!["users:read".to_string()],
    vec!["admin".to_string()],
    None
).await?;
```

---

### Method: `validate_token()`

**Signature**: `pub async fn validate_token(&self, token: &str) -> Result<JwtClaims, JwtError>`

**Purpose**: Validate and decode JWT

**Validation Checks**:
1. Signature verification (RS256)
2. Issuer validation
3. Audience validation
4. Expiration check
5. Not-before check

**Example**:
```rust
match jwt_service.validate_token(token).await {
    Ok(claims) => {
        // Token valid
        let user_id = Uuid::parse_str(&claims.sub)?;
    }
    Err(JwtError::TokenExpired) => {
        // Token expired
    }
    Err(_) => {
        // Invalid token
    }
}
```

---

### Method: `extract_claims_unsafe()`

**Signature**: `pub fn extract_claims_unsafe(&self, token: &str) -> Result<JwtClaims, JwtError>`

**Purpose**: Extract claims without validation (for introspection)

**Use Case**: Token introspection endpoint

**Warning**: Does NOT validate signature or expiration

---

### Method: `is_token_expired()`

**Signature**: `pub fn is_token_expired(&self, claims: &JwtClaims) -> bool`

**Purpose**: Check if token is expired

---

## Security Considerations

### 1. RS256 Algorithm

**Why RS256?**
- Asymmetric signing
- Public key can be distributed
- Private key never leaves server
- Industry standard

### 2. Token Lifetime

**Default**: 15 minutes for access tokens

**Rationale**: Short-lived tokens reduce impact of compromise

### 3. Signature Validation

**Critical**: Always validate signature before trusting claims

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `jsonwebtoken` | JWT operations |
| `serde` | Serialization |
| `chrono` | Timestamps |

---

## Related Files

- [keys.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-crypto/keys.md) - Key management

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 206  
**Security Level**: CRITICAL
