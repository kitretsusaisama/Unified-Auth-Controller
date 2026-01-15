# models/token.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/token.rs`  
**Crate**: `auth-core`  
**Module**: `models::token`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Contains JWT claims and refresh token structures

## Purpose

Defines token-related domain models for JWT-based authentication, including refresh tokens, access tokens, and JWT claims.

### Problem It Solves

- Type-safe representation of authentication tokens
- Enforces token structure and validation
- Manages token lifecycle (creation, expiration, revocation)
- Supports token rotation for security

---

## Detailed Code Breakdown

### Struct: `RefreshToken`

**Purpose**: Represents a long-lived refresh token for obtaining new access tokens

**Fields**:

| Field | Type | Description | Security Notes |
|-------|------|-------------|----------------|
| `id` | `Uuid` | Unique token identifier | Primary key |
| `user_id` | `Uuid` | Owner of the token | Foreign key to users table |
| `tenant_id` | `Uuid` | Tenant context | Multi-tenancy isolation |
| `token_family` | `Uuid` | Token rotation family | Detects token reuse attacks |
| `token_hash` | `String` | SHA-256 hash of token | **CRITICAL**: Never store plaintext |
| `device_fingerprint` | `Option<String>` | Device identifier | Security: Detect token theft |
| `user_agent` | `Option<String>` | Browser/client info | Audit trail |
| `ip_address` | `Option<String>` | Client IP address | Security: Geo-location checks |
| `expires_at` | `DateTime<Utc>` | Expiration timestamp | Typically 7 days |
| `revoked_at` | `Option<DateTime<Utc>>` | Revocation timestamp | Manual revocation |
| `revoked_reason` | `Option<String>` | Why revoked | Audit trail |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Audit trail |

**Security Features**:

1. **Token Hashing**: Only SHA-256 hash stored, never plaintext
   ```rust
   let token_hash = sha256(refresh_token);
   ```

2. **Token Family**: Enables rotation detection
   - All tokens in a family share same `token_family` UUID
   - If old token reused, entire family revoked (token theft detected)

3. **Device Fingerprinting**: Detects token theft across devices
   - Hash of: User-Agent + Screen Resolution + Timezone + Language
   - If fingerprint changes, require re-authentication

4. **Expiration**: Automatic expiration after 7 days
   - Reduces window for token theft
   - Forces periodic re-authentication

**Usage**:
```rust
let refresh_token = RefreshToken {
    id: Uuid::new_v4(),
    user_id: user.id,
    tenant_id: tenant.id,
    token_family: Uuid::new_v4(),
    token_hash: sha256(&random_token),
    device_fingerprint: Some(fingerprint),
    user_agent: Some(request.user_agent),
    ip_address: Some(request.ip),
    expires_at: Utc::now() + Duration::days(7),
    revoked_at: None,
    revoked_reason: None,
    created_at: Utc::now(),
};
```

---

### Struct: `AccessToken`

**Purpose**: Represents a short-lived JWT access token

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `token` | `String` | JWT token string (signed) |
| `token_type` | `String` | Always "Bearer" |
| `expires_in` | `u64` | Seconds until expiration (900 = 15 min) |
| `scope` | `Option<String>` | OAuth scopes (optional) |

**Example**:
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 900,
  "scope": "read:profile write:profile"
}
```

**Security**:
- Short expiration (15 minutes) limits damage from token theft
- Stateless validation (no database lookup required)
- Signed with RS256 (asymmetric cryptography)

---

### Struct: `TokenPair`

**Purpose**: Represents both access and refresh tokens issued together

**Fields**:
- `access_token`: AccessToken (JWT for API access)
- `refresh_token`: String (opaque token for renewal)

**Usage**:
```rust
let token_pair = TokenPair {
    access_token: AccessToken {
        token: jwt_string,
        token_type: "Bearer".to_string(),
        expires_in: 900,
        scope: None,
    },
    refresh_token: refresh_token_string,
};

// Return to client
Json(token_pair)
```

**Security**:
- Access token: Short-lived, stateless
- Refresh token: Long-lived, stateful (database-backed)
- Separation of concerns: Different security models

---

### Struct: `Claims`

**Purpose**: JWT payload containing user identity and authorization data

**Fields**:

| Field | Type | Description | JWT Standard |
|-------|------|-------------|--------------|
| `sub` | `String` | Subject (user ID) | ✅ RFC 7519 |
| `iss` | `String` | Issuer ("auth-service") | ✅ RFC 7519 |
| `aud` | `String` | Audience ("auth-service") | ✅ RFC 7519 |
| `exp` | `i64` | Expiration (Unix timestamp) | ✅ RFC 7519 |
| `iat` | `i64` | Issued At (Unix timestamp) | ✅ RFC 7519 |
| `nbf` | `i64` | Not Before (Unix timestamp) | ✅ RFC 7519 |
| `jti` | `String` | JWT ID (unique identifier) | ✅ RFC 7519 |
| `tenant_id` | `String` | Tenant context | Custom claim |
| `permissions` | `Vec<String>` | User permissions | Custom claim |
| `roles` | `Vec<String>` | User roles | Custom claim |

**Standard Claims (RFC 7519)**:

1. **`sub` (Subject)**: User identifier
   - Example: `"550e8400-e29b-41d4-a716-446655440000"`
   - Used to identify token owner

2. **`iss` (Issuer)**: Token issuer
   - Example: `"auth-service"`
   - Validates token origin

3. **`aud` (Audience)**: Intended recipient
   - Example: `"auth-service"`
   - Prevents token misuse across services

4. **`exp` (Expiration)**: Expiration time
   - Example: `1705234567` (Unix timestamp)
   - Automatic token invalidation

5. **`iat` (Issued At)**: Issuance time
   - Example: `1705233667`
   - Audit trail

6. **`nbf` (Not Before)**: Activation time
   - Example: `1705233667`
   - Prevents premature use

7. **`jti` (JWT ID)**: Unique token ID
   - Example: `"a1b2c3d4-e5f6-7890-abcd-ef1234567890"`
   - Enables token revocation

**Custom Claims**:

1. **`tenant_id`**: Multi-tenancy support
   - Isolates data by tenant
   - Required for all operations

2. **`permissions`**: Fine-grained permissions
   - Example: `["users:read:tenant", "roles:write:tenant"]`
   - RBAC/ABAC enforcement

3. **`roles`**: User roles
   - Example: `["UserManager", "Auditor"]`
   - Coarse-grained authorization

**Example JWT**:
```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "auth-service",
  "aud": "auth-service",
  "exp": 1705234567,
  "iat": 1705233667,
  "nbf": 1705233667,
  "jti": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "tenant_id": "tenant-uuid",
  "permissions": ["users:read:tenant"],
  "roles": ["UserManager"]
}
```

**Validation**:
```rust
// Validate expiration
if claims.exp < Utc::now().timestamp() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// Validate issuer
if claims.iss != "auth-service" {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Invalid });
}

// Validate audience
if claims.aud != "auth-service" {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Invalid });
}
```

---

## Token Lifecycle

### 1. Token Issuance

```rust
// Generate access token
let claims = Claims {
    sub: user.id.to_string(),
    iss: "auth-service".to_string(),
    aud: "auth-service".to_string(),
    exp: (Utc::now() + Duration::minutes(15)).timestamp(),
    iat: Utc::now().timestamp(),
    nbf: Utc::now().timestamp(),
    jti: Uuid::new_v4().to_string(),
    tenant_id: tenant.id.to_string(),
    permissions: user_permissions,
    roles: user_roles,
};

let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;

// Generate refresh token
let refresh_token = generate_random_token();
let refresh_token_hash = sha256(&refresh_token);

// Store in database
save_refresh_token(RefreshToken {
    token_hash: refresh_token_hash,
    expires_at: Utc::now() + Duration::days(7),
    // ...
})?;
```

### 2. Token Validation

```rust
// Decode and verify JWT
let token_data = decode::<Claims>(
    &jwt,
    &decoding_key,
    &Validation::new(Algorithm::RS256)
)?;

// Check expiration
if token_data.claims.exp < Utc::now().timestamp() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// Check revocation
if is_revoked(&token_data.claims.jti)? {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Revoked });
}
```

### 3. Token Refresh

```rust
// Hash provided refresh token
let token_hash = sha256(&refresh_token);

// Find in database
let stored_token = find_refresh_token(&token_hash)?;

// Check expiration
if stored_token.expires_at < Utc::now() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// Generate new tokens
let new_access_token = issue_access_token(user)?;
let new_refresh_token = issue_refresh_token(user)?;

// Revoke old refresh token
revoke_refresh_token(&stored_token.id)?;

// Return new pair
Ok(TokenPair { new_access_token, new_refresh_token })
```

### 4. Token Revocation

```rust
// Revoke specific token
revoke_refresh_token(token_id, "User logout")?;

// Revoke all user tokens
revoke_all_user_tokens(user_id, "Password changed")?;

// Revoke token family (theft detected)
revoke_token_family(token_family, "Token reuse detected")?;
```

---

## Security Considerations

### Refresh Token Security

1. **Hashed Storage**: Only SHA-256 hash stored
   - Prevents token theft from database breach
   - Rainbow table attacks ineffective (random tokens)

2. **Single-Use with Rotation**: Each refresh generates new tokens
   - Old refresh token immediately revoked
   - Detects token replay attacks

3. **Token Family**: Detects token theft
   - If old token reused, entire family revoked
   - Alerts user of potential compromise

4. **Device Fingerprinting**: Binds token to device
   - If fingerprint changes, require re-authentication
   - Prevents cross-device token theft

### Access Token Security

1. **Short Expiration**: 15 minutes
   - Limits damage from token theft
   - Forces periodic refresh

2. **Stateless Validation**: No database lookup
   - High performance
   - Scales horizontally

3. **RS256 Signing**: Asymmetric cryptography
   - Private key never leaves auth service
   - Public key distributed for validation
   - Prevents token forgery

4. **Revocation List**: For compromised tokens
   - `jti` checked against revocation list
   - TTL matches token expiration

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `chrono` | Timestamp handling |
| `serde` | JSON serialization |
| `uuid` | Unique identifiers |

### Internal Dependencies

- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - Token generation/validation
- [auth-crypto](file:///c:/Users/Victo/Downloads/sso/crates/auth-crypto) - JWT signing, SHA-256 hashing

---

## Related Files

- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - Token operations
- [repositories/token_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/token_repository.rs) - Token persistence
- [handlers/auth.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/handlers/auth.rs) - Token issuance endpoints

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 49  
**Security Level**: CRITICAL
