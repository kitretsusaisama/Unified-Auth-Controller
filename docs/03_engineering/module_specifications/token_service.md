---
title: Token Service Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Module Specification
crate: auth-core
---

# Token Service Specification

> [!NOTE]
> **Module**: `auth-core::services::token_service`  
> **Responsibility**: JWT token lifecycle management (issuance, validation, revocation, rotation)

---

## 1. Overview

### 1.1 Purpose

The **Token Service** manages the complete lifecycle of authentication tokens, including access tokens (JWT) and refresh tokens (opaque, database-backed). It handles token issuance, validation, refresh, and revocation.

### 1.2 Location

- **Crate**: `auth-core`
- **Module**: `src/services/token_service.rs`
- **Dependencies**: `auth-db` (token repositories), `jsonwebtoken`, `auth-crypto`

---

## 2. Public API

### 2.1 Traits

#### TokenProvider

```rust
#[async_trait]
pub trait TokenProvider: Send + Sync {
    async fn issue_access_token(&self, claims: Claims) 
        -> Result<AccessToken, AuthError>;
    
    async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) 
        -> Result<RefreshToken, AuthError>;
    
    async fn validate_access_token(&self, token: &str) 
        -> Result<Claims, AuthError>;
    
    async fn refresh_access_token(&self, refresh_token: &str) 
        -> Result<(AccessToken, RefreshToken), AuthError>;
    
    async fn revoke_token(&self, token_id: &str) 
        -> Result<(), AuthError>;
}
```

---

### 2.2 Service

#### TokenEngine

```rust
pub struct TokenEngine {
    access_token_ttl: Duration,
    refresh_token_ttl: Duration,
    signing_key: EncodingKey,
    validation_key: DecodingKey,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    revoked_token_repo: Arc<dyn RevokedTokenRepository>,
}
```

---

### 2.3 Models

#### AccessToken

```rust
pub struct AccessToken {
    pub token: String,           // JWT string
    pub expires_at: DateTime<Utc>,
}
```

#### RefreshToken

```rust
pub struct RefreshToken {
    pub token_hash: String,      // Opaque token (hashed)
    pub expires_at: DateTime<Utc>,
}
```

#### Claims

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,             // Subject (user_id)
    pub iss: String,             // Issuer
    pub aud: String,             // Audience
    pub exp: i64,                // Expiration time
    pub iat: i64,                // Issued at
    pub nbf: i64,                // Not before
    pub jti: String,             // JWT ID (unique)
    pub tenant_id: String,       // Tenant ID
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}
```

---

## 3. Operations

### 3.1 Issue Access Token

**Method**: `issue_access_token(claims: Claims) -> Result<AccessToken>`

**Purpose**: Create a signed JWT access token.

**Algorithm**: RS256 (RSA with SHA-256)

**TTL**: 15 minutes (configurable)

**Flow**:
1. Set expiration time (now + 15 min)
2. Sign claims with private key (RS256)
3. Return JWT string

**Invariants**:
- Token is signed with RS256
- Expiration is always set
- JWT ID (jti) is unique

---

### 3.2 Issue Refresh Token

**Method**: `issue_refresh_token(user_id: Uuid, tenant_id: Uuid) -> Result<RefreshToken>`

**Purpose**: Create an opaque refresh token stored in database.

**TTL**: 30 days (configurable)

**Flow**:
1. Generate random token (32 bytes, base64)
2. Hash token (SHA-256)
3. Store hash in database with user_id, tenant_id, expiration
4. Return original token (unhashed) to client

**Invariants**:
- Token is cryptographically random
- Only hash is stored in database
- Token is single-use (rotation on refresh)

---

### 3.3 Validate Access Token

**Method**: `validate_access_token(token: &str) -> Result<Claims>`

**Purpose**: Verify JWT signature and extract claims.

**Flow**:
1. Decode JWT with public key
2. Verify signature (RS256)
3. Check expiration (exp claim)
4. Check not-before (nbf claim)
5. Check revocation list (jti)
6. Return claims

**Invariants**:
- Signature must be valid
- Token must not be expired
- Token must not be revoked

**Error Cases**:
- `TokenError::InvalidSignature`: Signature verification failed
- `TokenError::Expired`: Token is past expiration
- `TokenError::Revoked`: Token is in revocation list

---

### 3.4 Refresh Access Token

**Method**: `refresh_access_token(refresh_token: &str) -> Result<(AccessToken, RefreshToken)>`

**Purpose**: Exchange refresh token for new access token + new refresh token.

**Flow**:
1. Hash provided refresh token
2. Look up hash in database
3. Verify token exists and not expired
4. Extract user_id, tenant_id from database record
5. Delete old refresh token (single-use)
6. Issue new access token
7. Issue new refresh token (rotation)
8. Return both tokens

**Invariants**:
- Old refresh token is deleted (single-use)
- New refresh token is issued (rotation)
- Access token contains same user_id, tenant_id

**Error Cases**:
- `TokenError::Invalid`: Refresh token not found
- `TokenError::Expired`: Refresh token expired
- `TokenError::Revoked`: Refresh token revoked

---

### 3.5 Revoke Token

**Method**: `revoke_token(token_id: &str) -> Result<()>`

**Purpose**: Add token to revocation list.

**Flow**:
1. Extract JWT ID (jti) from token
2. Add jti to revoked tokens table
3. Set expiration (same as original token)

**Invariants**:
- Revoked tokens are checked on validation
- Revocation list is pruned (expired tokens removed)

---

## 4. Token Types

### 4.1 Access Token (JWT)

**Format**: JSON Web Token (JWT)

**Algorithm**: RS256 (asymmetric)

**Structure**:
```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "iss": "auth-service",
  "aud": "auth-service",
  "exp": 1641900900,
  "iat": 1641900000,
  "nbf": 1641900000,
  "jti": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "tenant_id": "tenant-uuid",
  "permissions": ["read:users", "write:users"],
  "roles": ["admin"]
}
```

**Characteristics**:
- **Stateless**: No database lookup required for validation
- **Short-lived**: 15 minutes
- **Self-contained**: Contains all necessary claims
- **Revocable**: Via revocation list (jti)

---

### 4.2 Refresh Token (Opaque)

**Format**: Random base64 string

**Storage**: Database-backed

**Structure** (in database):
```sql
CREATE TABLE refresh_tokens (
    id BINARY(16) PRIMARY KEY,
    token_hash VARCHAR(64) NOT NULL,
    user_id BINARY(16) NOT NULL,
    tenant_id BINARY(16) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_token_hash (token_hash),
    INDEX idx_user_id (user_id)
);
```

**Characteristics**:
- **Stateful**: Requires database lookup
- **Long-lived**: 30 days
- **Single-use**: Deleted on refresh (rotation)
- **Revocable**: Delete from database

---

## 5. Security Considerations

### 5.1 Cryptography

**Access Token Signing**:
- Algorithm: RS256 (RSA + SHA-256)
- Key Size: 2048 bits minimum
- Key Rotation: Supported (multiple keys)

**Refresh Token Generation**:
- Random: `OsRng` (cryptographically secure)
- Length: 32 bytes (256 bits)
- Encoding: Base64

**Refresh Token Storage**:
- Hashed: SHA-256
- Never store plain token

---

### 5.2 Token Rotation

**Refresh Token Rotation**:
- Old refresh token deleted on use
- New refresh token issued
- Prevents replay attacks

**Access Token Rotation**:
- New access token on every refresh
- Old access token expires naturally

---

### 5.3 Revocation

**Access Token Revocation**:
- Add jti to revocation list
- Checked on every validation
- Pruned after expiration

**Refresh Token Revocation**:
- Delete from database
- Immediate effect

---

## 6. Performance Characteristics

### 6.1 Latency

| Operation | Target | Typical | Notes |
|-----------|--------|---------|-------|
| Issue Access Token | <5ms | 2ms | In-memory signing |
| Issue Refresh Token | <20ms | 10ms | Database insert |
| Validate Access Token | <5ms | 2ms | In-memory verification |
| Refresh Access Token | <30ms | 15ms | DB lookup + delete + insert |
| Revoke Token | <20ms | 10ms | Database insert |

### 6.2 Caching

**Public Key Caching**:
- Cache public keys for validation
- TTL: 1 hour
- Reduces key loading overhead

**Revocation List Caching**:
- Cache revoked token IDs
- TTL: 5 minutes
- Reduces database queries

---

## 7. Failure Modes

### 7.1 Key Loading Failure

**Scenario**: Private/public key cannot be loaded

**Behavior**:
- Service fails to start
- Return error immediately

**Recovery**:
- Fix key configuration
- Restart service

---

### 7.2 Database Failure

**Scenario**: Database unavailable during refresh token operation

**Behavior**:
- Return `DatabaseError`
- No tokens issued

**Recovery**:
- Client retries
- Circuit breaker (future)

---

### 7.3 Token Expiration

**Scenario**: Access token expired

**Behavior**:
- Return `TokenError::Expired`
- Client must refresh

**Recovery**:
- Client uses refresh token to get new access token

---

## 8. Testing Strategy

### 8.1 Unit Tests

**Test Cases**:
- ✅ Issue access token with valid claims
- ✅ Issue refresh token
- ✅ Validate access token with valid signature
- ✅ Validate access token with invalid signature (should fail)
- ✅ Validate expired access token (should fail)
- ✅ Refresh access token with valid refresh token
- ✅ Refresh access token with invalid refresh token (should fail)
- ✅ Revoke access token
- ✅ Validate revoked access token (should fail)

---

### 8.2 Property-Based Tests

**Properties**:
- Issued access token always validates successfully (before expiration)
- Expired access token never validates
- Refresh token is single-use (second use fails)
- Revoked token never validates

**Tool**: `proptest`

---

## 9. Configuration

### 9.1 Token Configuration

```toml
[security.tokens]
access_token_ttl_minutes = 15
refresh_token_ttl_days = 30
algorithm = "RS256"
issuer = "auth-service"
audience = "auth-service"
```

### 9.2 Key Configuration

```toml
[security.keys]
private_key_path = "/path/to/private_key.pem"
public_key_path = "/path/to/public_key.pem"
```

---

## 10. Examples

### 10.1 Issue Tokens

```rust
use auth_core::services::token_service::{TokenEngine, Claims};
use uuid::Uuid;

let token_engine = TokenEngine::new(/* config */);

// Issue access token
let claims = Claims {
    sub: user_id.to_string(),
    iss: "auth-service".to_string(),
    aud: "auth-service".to_string(),
    exp: (Utc::now() + Duration::minutes(15)).timestamp(),
    iat: Utc::now().timestamp(),
    nbf: Utc::now().timestamp(),
    jti: Uuid::new_v4().to_string(),
    tenant_id: tenant_id.to_string(),
    permissions: vec![],
    roles: vec![],
};

let access_token = token_engine.issue_access_token(claims).await?;

// Issue refresh token
let refresh_token = token_engine.issue_refresh_token(user_id, tenant_id).await?;
```

### 10.2 Validate and Refresh

```rust
// Validate access token
let claims = token_engine.validate_access_token(&access_token.token).await?;

// Refresh access token
let (new_access, new_refresh) = token_engine
    .refresh_access_token(&refresh_token.token_hash)
    .await?;
```

---

**Document Status**: Active  
**Next Review**: 2026-04-12 (3 months)  
**Owner**: Engineering Team
