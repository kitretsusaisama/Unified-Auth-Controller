# services/token_service.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/token_service.rs`  
**Crate**: `auth-core`  
**Module**: `services::token_service`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - JWT token management and validation

## Purpose

Comprehensive JWT token management service providing access token generation, refresh token rotation, token validation, revocation, and introspection using RS256 signing.

### Problem It Solves

- JWT token generation and validation
- Refresh token rotation with family tracking
- Token revocation and blacklisting
- Token introspection
- Breach detection
- Secure token lifecycle management

---

## Detailed Code Breakdown

### Trait: `RefreshTokenStore`

**Purpose**: Persistence abstraction for refresh tokens

**Methods**:
```rust
async fn create(&self, token: RefreshToken) -> Result<(), AuthError>;
async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshToken>, AuthError>;
async fn revoke(&self, token_id: Uuid) -> Result<(), AuthError>;
async fn revoke_family(&self, family_id: Uuid) -> Result<(), AuthError>;
```

---

### Trait: `RevokedTokenStore`

**Purpose**: Access token blacklist

**Methods**:
```rust
async fn add_to_blacklist(&self, jti: Uuid, user_id: Uuid, tenant_id: Uuid, expires_at: DateTime<Utc>) -> Result<(), AuthError>;
async fn is_revoked(&self, jti: Uuid) -> Result<bool, AuthError>;
```

---

### Trait: `TokenProvider`

**Purpose**: Main token service interface

**Methods**:

```rust
async fn issue_access_token(&self, claims: Claims) -> Result<AccessToken, AuthError>;
async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> Result<RefreshToken, AuthError>;
async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
async fn revoke_token(&self, token_id: Uuid, user_id: Uuid, tenant_id: Uuid) -> Result<(), AuthError>;
async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResponse, AuthError>;
```

---

### Struct: `TokenEngine`

**Purpose**: Main token service implementation

**Fields**:
- `jwt_service`: `JwtService` - JWT encoding/decoding
- `revoked_token_store`: `Arc<dyn RevokedTokenStore>` - Blacklist
- `refresh_token_store`: `Arc<dyn RefreshTokenStore>` - Refresh tokens

---

### Struct: `TokenIntrospectionResponse`

**Purpose**: OAuth 2.0 token introspection response

**Fields**:
- `active`: Token validity
- `scope`, `client_id`, `username`: Token metadata
- `token_type`, `exp`, `iat`, `nbf`: Token properties
- `sub`, `aud`, `iss`, `jti`: JWT claims

---

## Token Operations

### Method: `issue_access_token()`

**Signature**: `async fn issue_access_token(&self, claims: Claims) -> Result<AccessToken, AuthError>`

**Purpose**: Generate signed JWT access token

**Process**:
1. Create JWT claims
2. Sign with RS256 private key
3. Return access token

**Example**:
```rust
let claims = Claims {
    sub: user_id.to_string(),
    iss: "auth.example.com".to_string(),
    aud: "api.example.com".to_string(),
    exp: (Utc::now() + Duration::minutes(15)).timestamp(),
    iat: Utc::now().timestamp(),
    nbf: Utc::now().timestamp(),
    jti: Uuid::new_v4().to_string(),
    tenant_id: tenant_id.to_string(),
    permissions: vec!["users:read".to_string()],
    roles: vec!["user".to_string()],
};

let access_token = token_engine.issue_access_token(claims).await?;
```

---

### Method: `issue_refresh_token()`

**Signature**: `async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> Result<RefreshToken, AuthError>`

**Purpose**: Generate refresh token with family tracking

**Process**:
1. Generate random token
2. Hash token (SHA-256)
3. Create family ID (for rotation tracking)
4. Store in database
5. Return plaintext token

**Token Family**: All rotated tokens share same family_id for breach detection

**Example**:
```rust
let refresh_token = token_engine.issue_refresh_token(user_id, tenant_id).await?;
// Returns: RefreshToken { token: "...", expires_at: ... }
```

---

### Method: `validate_token()`

**Signature**: `async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>`

**Purpose**: Validate and decode JWT

**Validation Steps**:
1. Verify signature (RS256)
2. Check expiration
3. Validate issuer/audience
4. Check revocation blacklist

**Example**:
```rust
match token_engine.validate_token(token_str).await {
    Ok(claims) => {
        // Token valid, use claims
        let user_id = Uuid::parse_str(&claims.sub)?;
    }
    Err(AuthError::TokenError { kind: TokenErrorKind::Expired }) => {
        // Token expired
    }
    Err(AuthError::TokenError { kind: TokenErrorKind::Revoked }) => {
        // Token revoked
    }
    Err(_) => {
        // Invalid token
    }
}
```

---

### Method: `refresh_tokens()`

**Signature**: `async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair, AuthError>`

**Purpose**: Rotate refresh token and issue new access token

**Process**:

#### 1. Hash and Find Token
```rust
let token_hash = sha256(refresh_token);
let stored_token = self.refresh_token_store.find_by_hash(&token_hash).await?;
```

#### 2. Validate Token
```rust
if stored_token.expires_at < Utc::now() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

if stored_token.revoked_at.is_some() {
    // BREACH DETECTED: Revoke entire family
    self.refresh_token_store.revoke_family(stored_token.family_id).await?;
    return Err(AuthError::TokenError { kind: TokenErrorKind::Revoked });
}
```

#### 3. Revoke Current Token
```rust
self.refresh_token_store.revoke(stored_token.id).await?;
```

#### 4. Issue New Tokens
```rust
let new_access_token = self.issue_access_token(claims).await?;
let new_refresh_token = self.issue_refresh_token(user_id, tenant_id).await?;

Ok(TokenPair {
    access_token: new_access_token,
    refresh_token: new_refresh_token,
})
```

---

### Method: `revoke_token()`

**Signature**: `async fn revoke_token(&self, token_id: Uuid, user_id: Uuid, tenant_id: Uuid) -> Result<(), AuthError>`

**Purpose**: Add access token to blacklist

**Use Cases**:
- User logout
- Admin revocation
- Security incident

**Example**:
```rust
// On logout
let claims = extract_claims(token)?;
let jti = Uuid::parse_str(&claims.jti)?;
token_engine.revoke_token(jti, user_id, tenant_id).await?;
```

---

### Method: `introspect_token()`

**Signature**: `async fn introspect_token(&self, token: &str) -> Result<TokenIntrospectionResponse, AuthError>`

**Purpose**: OAuth 2.0 token introspection

**Returns**: Token metadata without validating signature (for debugging)

**Example**:
```rust
let introspection = token_engine.introspect_token(token).await?;
if introspection.active {
    println!("Token expires at: {:?}", introspection.exp);
}
```

---

## Token Rotation and Breach Detection

### Refresh Token Family

**Concept**: All rotated tokens share a `family_id`

```
Initial Token (family_id: abc123)
  ↓ rotate
Token 2 (family_id: abc123, parent: token1)
  ↓ rotate
Token 3 (family_id: abc123, parent: token2)
```

### Breach Detection

**Scenario**: Attacker steals Token 2, user already rotated to Token 3

```rust
// User uses Token 3 (valid)
refresh_tokens(token3) // ✅ Success

// Attacker uses Token 2 (already revoked)
refresh_tokens(token2) // ❌ Breach detected!
// → Revoke entire family (token1, token2, token3)
// → Force user re-authentication
```

**Implementation**:
```rust
if stored_token.revoked_at.is_some() {
    // Token already used - BREACH!
    self.refresh_token_store.revoke_family(stored_token.family_id).await?;
    
    // Alert security team
    alert_security_breach(user_id, "Refresh token reuse detected").await;
    
    return Err(AuthError::TokenError { kind: TokenErrorKind::Revoked });
}
```

---

## In-Memory Implementations

### InMemoryRefreshTokenStore

**Purpose**: Testing/development implementation

```rust
pub struct InMemoryRefreshTokenStore {
    tokens: RwLock<HashMap<String, RefreshToken>>,
}
```

### InMemoryRevokedTokenStore

**Purpose**: Testing/development blacklist

```rust
pub struct InMemoryRevokedTokenStore {
    revoked: RwLock<HashSet<Uuid>>,
}
```

---

## Usage Examples

### Example 1: Login Flow

```rust
pub async fn login(
    email: String,
    password: String,
    token_engine: &TokenEngine,
) -> Result<TokenPair> {
    // Authenticate user
    let user = authenticate(email, password).await?;
    
    // Create claims
    let claims = Claims {
        sub: user.id.to_string(),
        iss: "auth.example.com".to_string(),
        aud: "api.example.com".to_string(),
        exp: (Utc::now() + Duration::minutes(15)).timestamp(),
        iat: Utc::now().timestamp(),
        nbf: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
        tenant_id: user.tenant_id.to_string(),
        permissions: get_user_permissions(user.id).await?,
        roles: get_user_roles(user.id).await?,
    };
    
    // Issue tokens
    let access_token = token_engine.issue_access_token(claims).await?;
    let refresh_token = token_engine.issue_refresh_token(user.id, user.tenant_id).await?;
    
    Ok(TokenPair { access_token, refresh_token })
}
```

---

### Example 2: Token Refresh

```rust
pub async fn refresh(
    refresh_token_str: String,
    token_engine: &TokenEngine,
) -> Result<TokenPair> {
    // Rotate tokens
    let new_tokens = token_engine.refresh_tokens(&refresh_token_str).await?;
    
    Ok(new_tokens)
}
```

---

### Example 3: Logout

```rust
pub async fn logout(
    access_token: String,
    refresh_token: String,
    token_engine: &TokenEngine,
) -> Result<()> {
    // Extract claims
    let claims = extract_claims_unsafe(&access_token)?;
    let jti = Uuid::parse_str(&claims.jti)?;
    let user_id = Uuid::parse_str(&claims.sub)?;
    let tenant_id = Uuid::parse_str(&claims.tenant_id)?;
    
    // Revoke access token
    token_engine.revoke_token(jti, user_id, tenant_id).await?;
    
    // Revoke refresh token
    let refresh_hash = sha256(&refresh_token);
    if let Some(stored) = token_engine.refresh_token_store.find_by_hash(&refresh_hash).await? {
        token_engine.refresh_token_store.revoke(stored.id).await?;
    }
    
    Ok(())
}
```

---

### Example 4: Middleware Validation

```rust
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from header
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Validate token
    let claims = state.token_engine
        .validate_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add claims to request extensions
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}
```

---

## Security Considerations

### 1. Token Expiration

**Access Token**: Short-lived (15 minutes)
**Refresh Token**: Long-lived (30 days)

### 2. Signature Algorithm

**RS256**: Asymmetric signing
- Private key signs tokens
- Public key verifies tokens
- Public key can be distributed

### 3. Token Rotation

**Always rotate** refresh tokens on use to detect breaches

### 4. Revocation

**Access tokens**: Blacklist (short TTL makes this feasible)
**Refresh tokens**: Database revocation

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `jsonwebtoken` | JWT encoding/decoding (production) |
| `uuid` | Token identifiers |
| `chrono` | Timestamps |
| `sha2` | Token hashing |

### Internal Dependencies

- [models/token.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/token.md) - Token models
- [error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [repositories/refresh_token_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/refresh_token_repository.md) - Refresh token persistence
- [repositories/revoked_token_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/revoked_token_repository.md) - Token blacklist

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 436  
**Security Level**: CRITICAL
