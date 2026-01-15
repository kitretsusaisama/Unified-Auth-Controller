# repositories/refresh_token_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/refresh_token_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::refresh_token_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ✅ **YES** - Token security and breach detection

## Purpose

Implements refresh token persistence with token family tracking and breach detection, providing secure long-lived authentication with automatic revocation on suspicious activity.

### Problem It Solves

- Secure refresh token storage with SHA-256 hashing
- Token family tracking for rotation detection
- Breach detection when revoked tokens are reused
- Automatic family revocation on security incidents

---

## Detailed Code Breakdown

### Enum: `RefreshTokenError`

**Purpose**: Specific error types for refresh token operations

**Variants**:

| Variant | Description |
|---------|-------------|
| `DatabaseError(sqlx::Error)` | Database operation failed |
| `TokenNotFound` | Token doesn't exist |
| `TokenExpired` | Token past expiration |
| `TokenRevoked` | Token has been revoked |
| `FamilyBreachDetected` | Revoked token reused (security incident) |

---

### Struct: `RefreshTokenRecord`

**Purpose**: Database representation of refresh token

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Token identifier |
| `user_id` | `Uuid` | Owner user |
| `tenant_id` | `Uuid` | Tenant (multi-tenancy) |
| `token_family` | `Uuid` | Family for rotation tracking |
| `token_hash` | `String` | SHA-256 hash of token |
| `device_fingerprint` | `Option<String>` | Device identifier |
| `user_agent` | `Option<String>` | Browser user agent |
| `ip_address` | `Option<String>` | Client IP |
| `expires_at` | `DateTime<Utc>` | Expiration time |
| `revoked_at` | `Option<DateTime<Utc>>` | Revocation time |
| `revoked_reason` | `Option<String>` | Revocation reason |
| `created_at` | `DateTime<Utc>` | Creation time |

---

### Struct: `RefreshTokenRepository`

**Purpose**: MySQL/SQLite implementation with advanced security features

**Fields**:
- `pool`: `Pool<MySql>` - Database connection pool

---

## Core Methods

### Method: `create()`

**Signature**:
```rust
pub async fn create(
    &self,
    user_id: Uuid,
    tenant_id: Uuid,
    token_family: Uuid,
    token_hash: String,
    device_fingerprint: Option<String>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    expires_at: DateTime<Utc>,
) -> Result<RefreshTokenRecord, RefreshTokenError>
```

**Purpose**: Create new refresh token

**SQL Query**:
```sql
INSERT INTO refresh_tokens (
    id, user_id, tenant_id, token_family, token_hash,
    device_fingerprint, user_agent, ip_address, expires_at, created_at
) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

**Returns**: Created token record

**Example**:
```rust
let token_hash = hash_token(&raw_token);
let record = repo.create(
    user_id,
    tenant_id,
    Uuid::new_v4(), // New family
    token_hash,
    Some(device_fingerprint),
    Some(user_agent),
    Some(ip_address),
    Utc::now() + Duration::days(30),
).await?;
```

---

### Method: `find_by_token_hash()`

**Signature**: `pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<RefreshTokenRecord, RefreshTokenError>`

**Purpose**: Retrieve token by hash

**SQL Query**:
```sql
SELECT id, user_id, tenant_id, token_family, token_hash,
       device_fingerprint, user_agent, ip_address,
       expires_at, revoked_at, revoked_reason, created_at
FROM refresh_tokens
WHERE token_hash = ?
```

**Returns**: Token record or `TokenNotFound` error

---

### Method: `find_by_family()`

**Signature**: `pub async fn find_by_family(&self, token_family: Uuid) -> Result<Vec<RefreshTokenRecord>, RefreshTokenError>`

**Purpose**: Get all tokens in a family (for breach detection)

**SQL Query**:
```sql
SELECT * FROM refresh_tokens
WHERE token_family = ?
ORDER BY created_at DESC
```

**Use Cases**:
- Breach investigation
- Family revocation
- Token rotation audit

---

### Method: `find_by_user()`

**Signature**: `pub async fn find_by_user(&self, user_id: Uuid, tenant_id: Uuid) -> Result<Vec<RefreshTokenRecord>, RefreshTokenError>`

**Purpose**: Get all active tokens for user

**SQL Query**:
```sql
SELECT * FROM refresh_tokens
WHERE user_id = ? AND tenant_id = ?
  AND revoked_at IS NULL
  AND expires_at > ?
ORDER BY created_at DESC
```

**Use Cases**:
- "Active sessions" display
- "Logout from all devices"
- Security audit

---

### Method: `revoke_token()`

**Signature**: `pub async fn revoke_token(&self, token_id: Uuid, reason: Option<String>) -> Result<(), RefreshTokenError>`

**Purpose**: Revoke single token

**SQL Query**:
```sql
UPDATE refresh_tokens
SET revoked_at = ?, revoked_reason = ?
WHERE id = ? AND revoked_at IS NULL
```

**Returns**: Error if token not found or already revoked

**Example**:
```rust
// User logout
repo.revoke_token(token_id, Some("User logout".to_string())).await?;
```

---

### Method: `revoke_family()`

**Signature**: `pub async fn revoke_family(&self, token_family: Uuid, reason: String) -> Result<u64, RefreshTokenError>`

**Purpose**: Revoke entire token family (breach response)

**SQL Query**:
```sql
UPDATE refresh_tokens
SET revoked_at = ?, revoked_reason = ?
WHERE token_family = ? AND revoked_at IS NULL
```

**Returns**: Number of tokens revoked

**Use Cases**:
- Breach detection response
- Security incident
- Compromised device

**Example**:
```rust
// Breach detected
let revoked_count = repo.revoke_family(
    family_id,
    "Token reuse detected - possible breach".to_string()
).await?;

info!("Revoked {} tokens in family", revoked_count);
```

---

### Method: `detect_breach()`

**Signature**: `pub async fn detect_breach(&self, token_hash: &str) -> Result<Option<Uuid>, RefreshTokenError>`

**Purpose**: Check if token is revoked (indicates breach if reused)

**SQL Query**:
```sql
SELECT token_family
FROM refresh_tokens
WHERE token_hash = ? AND revoked_at IS NOT NULL
```

**Returns**: `Some(family_id)` if revoked token, `None` if not found

**Security Flow**:
```rust
// On token refresh attempt
if let Some(family_id) = repo.detect_breach(&token_hash).await? {
    // BREACH DETECTED: Revoked token being reused
    repo.revoke_family(family_id, "Breach detected".to_string()).await?;
    
    // Alert security team
    alert_security_team(user_id, "Token reuse detected").await;
    
    return Err(RefreshTokenError::FamilyBreachDetected);
}
```

---

## Token Family Tracking

### Concept

**Token Family**: Group of tokens created through rotation

**Example Flow**:
```
Login → Token A (family: F1)
  ↓
Refresh → Token B (family: F1, revoke A)
  ↓
Refresh → Token C (family: F1, revoke B)
  ↓
Refresh → Token D (family: F1, revoke C)
```

**Breach Scenario**:
```
Current: Token D (active)
Attacker uses: Token B (revoked)
  ↓
System detects: Token B is revoked
  ↓
Response: Revoke entire family F1 (including D)
  ↓
Result: Attacker AND legitimate user logged out
```

---

### Implementation

```rust
pub async fn rotate_refresh_token(
    repo: &RefreshTokenRepository,
    old_token_hash: &str,
) -> Result<RefreshTokenRecord, RefreshTokenError> {
    // 1. Find old token
    let old_token = repo.find_by_token_hash(old_token_hash).await?;
    
    // 2. Check if revoked (breach detection)
    if old_token.revoked_at.is_some() {
        // BREACH: Revoked token being reused
        repo.revoke_family(
            old_token.token_family,
            "Token reuse detected".to_string()
        ).await?;
        return Err(RefreshTokenError::FamilyBreachDetected);
    }
    
    // 3. Check expiration
    if old_token.expires_at < Utc::now() {
        return Err(RefreshTokenError::TokenExpired);
    }
    
    // 4. Revoke old token
    repo.revoke_token(old_token.id, Some("Rotated".to_string())).await?;
    
    // 5. Create new token in same family
    let new_token_hash = generate_token_hash();
    let new_token = repo.create(
        old_token.user_id,
        old_token.tenant_id,
        old_token.token_family, // SAME FAMILY
        new_token_hash,
        old_token.device_fingerprint,
        old_token.user_agent,
        old_token.ip_address,
        Utc::now() + Duration::days(30),
    ).await?;
    
    Ok(new_token)
}
```

---

## Database Schema

```sql
CREATE TABLE refresh_tokens (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    token_family CHAR(36) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    device_fingerprint VARCHAR(255),
    user_agent TEXT,
    ip_address VARCHAR(45),
    expires_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NULL,
    revoked_reason VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_user_id (user_id),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_token_family (token_family),
    INDEX idx_token_hash (token_hash),
    INDEX idx_expires_at (expires_at),
    INDEX idx_revoked_at (revoked_at),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
```

---

## Security Features

### 1. Token Hashing

**Never store plaintext tokens**:
```rust
use sha2::{Sha256, Digest};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### 2. Breach Detection

**Automatic family revocation**:
```rust
if let Some(family_id) = repo.detect_breach(&token_hash).await? {
    repo.revoke_family(family_id, "Breach detected".to_string()).await?;
    alert_security_team(user_id).await;
}
```

### 3. Cleanup Job

**Remove expired tokens**:
```rust
pub async fn cleanup_expired(&self) -> Result<u64, RefreshTokenError> {
    let result = sqlx::query!(
        "DELETE FROM refresh_tokens WHERE expires_at < ?",
        Utc::now()
    ).execute(&self.pool).await?;
    
    Ok(result.rows_affected())
}
```

**Schedule**:
```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        if let Err(e) = repo.cleanup_expired().await {
            error!("Cleanup failed: {}", e);
        }
    }
});
```

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_token_rotation(pool: MySqlPool) {
    let repo = RefreshTokenRepository::new(pool);
    
    // Create initial token
    let family_id = Uuid::new_v4();
    let token1 = repo.create(
        user_id, tenant_id, family_id,
        "hash1".to_string(), None, None, None,
        Utc::now() + Duration::days(30)
    ).await.unwrap();
    
    // Rotate
    repo.revoke_token(token1.id, Some("Rotated".to_string())).await.unwrap();
    let token2 = repo.create(
        user_id, tenant_id, family_id,
        "hash2".to_string(), None, None, None,
        Utc::now() + Duration::days(30)
    ).await.unwrap();
    
    // Verify token1 is revoked
    let found = repo.find_by_token_hash("hash1").await.unwrap();
    assert!(found.revoked_at.is_some());
}

#[sqlx::test]
async fn test_breach_detection(pool: MySqlPool) {
    let repo = RefreshTokenRepository::new(pool);
    let family_id = Uuid::new_v4();
    
    // Create and revoke token
    let token = repo.create(...).await.unwrap();
    repo.revoke_token(token.id, Some("Test".to_string())).await.unwrap();
    
    // Detect breach
    let breach = repo.detect_breach(&token.token_hash).await.unwrap();
    assert_eq!(breach, Some(family_id));
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database operations |
| `uuid` | Token identifiers |
| `chrono` | Timestamps |
| `thiserror` | Error definitions |
| `async-trait` | Async trait support |

### Internal Dependencies

- [auth-core/models/token.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/token.md) - RefreshToken model
- [auth-core/services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - RefreshTokenStore trait

---

## Related Files

- [models/token.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/token.md) - Token models
- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Token service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 413  
**Security Level**: CRITICAL
