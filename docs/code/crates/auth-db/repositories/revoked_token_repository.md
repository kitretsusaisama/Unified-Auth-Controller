# repositories/revoked_token_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/revoked_token_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::revoked_token_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ✅ **YES** - Token blacklist and revocation

## Purpose

Implements token revocation blacklist for access tokens, enabling immediate token invalidation and preventing token reuse after logout or security incidents.

### Problem It Solves

- Immediate token revocation (logout)
- Token blacklist for compromised tokens
- Emergency user-wide token revocation
- Audit trail for token revocations
- Prevents token reuse attacks

---

## Detailed Code Breakdown

### Enum: `RevokedTokenError`

**Purpose**: Specific error types for token revocation

**Variants**:
- `DatabaseError(sqlx::Error)` - Database operation failed
- `TokenAlreadyRevoked` - Token already in blacklist

---

### Struct: `RevokedTokenRecord`

**Purpose**: Database representation of revoked token

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Record identifier |
| `token_jti` | `Uuid` | JWT ID (jti claim) |
| `user_id` | `Uuid` | Token owner |
| `tenant_id` | `Uuid` | Tenant context |
| `token_type` | `TokenType` | Access or Refresh |
| `revoked_at` | `DateTime<Utc>` | Revocation timestamp |
| `revoked_by` | `Option<Uuid>` | Who revoked (user/admin) |
| `revoked_reason` | `Option<String>` | Revocation reason |
| `expires_at` | `DateTime<Utc>` | Original token expiry |

---

### Enum: `TokenType`

**Purpose**: Token classification

**Variants**:
- `Access` - Short-lived access token
- `Refresh` - Long-lived refresh token

**Methods**:
```rust
fn to_str(&self) -> &'static str;
fn from_str(s: &str) -> Self;
```

---

### Struct: `RevokedTokenRepository`

**Purpose**: MySQL/SQLite implementation

**Fields**:
- `pool`: `Pool<MySql>` - Database connection pool

---

## Core Methods

### Method: `add_revoked_token()`

**Signature**:
```rust
pub async fn add_revoked_token(
    &self,
    token_jti: Uuid,
    user_id: Uuid,
    tenant_id: Uuid,
    token_type: TokenType,
    revoked_by: Option<Uuid>,
    revoked_reason: Option<String>,
    expires_at: DateTime<Utc>,
) -> Result<RevokedTokenRecord, RevokedTokenError>
```

**Purpose**: Add token to blacklist

**Process**:

1. **Check if already revoked**
   ```rust
   if self.is_token_revoked(token_jti).await? {
       return Err(RevokedTokenError::TokenAlreadyRevoked);
   }
   ```

2. **Insert revocation record**
   ```sql
   INSERT INTO revoked_tokens (
       id, token_jti, user_id, tenant_id, token_type,
       revoked_at, revoked_by, revoked_reason, expires_at
   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
   ```

**Returns**: Created revocation record

**Example**:
```rust
// Revoke access token on logout
let record = repo.add_revoked_token(
    token_jti,
    user_id,
    tenant_id,
    TokenType::Access,
    Some(user_id), // Self-revoked
    Some("User logout".to_string()),
    token_expires_at,
).await?;
```

---

### Method: `is_token_revoked()`

**Signature**: `pub async fn is_token_revoked(&self, token_jti: Uuid) -> Result<bool, RevokedTokenError>`

**Purpose**: Check if token is blacklisted (optimized for high throughput)

**SQL Query**:
```sql
SELECT COUNT(*) as count
FROM revoked_tokens
WHERE token_jti = ? AND expires_at > ?
```

**Optimization**: Uses COUNT for fast existence check

**Returns**: `true` if revoked, `false` otherwise

**Usage**:
```rust
// In token validation middleware
if repo.is_token_revoked(token_jti).await? {
    return Err(AuthError::TokenError {
        kind: TokenErrorKind::Revoked,
    });
}
```

---

### Method: `revoke_all_user_tokens()`

**Signature**:
```rust
pub async fn revoke_all_user_tokens(
    &self,
    user_id: Uuid,
    tenant_id: Uuid,
    revoked_by: Option<Uuid>,
    reason: String,
) -> Result<u64, RevokedTokenError>
```

**Purpose**: Emergency revocation of all user tokens

**Use Cases**:
- Account compromise
- Password change
- Security incident
- Admin action

**Implementation**: Adds marker record for bulk revocation

**Example**:
```rust
// After password change
let count = repo.revoke_all_user_tokens(
    user_id,
    tenant_id,
    Some(user_id),
    "Password changed".to_string(),
).await?;

info!("Revoked {} tokens for user", count);
```

---

### Method: `cleanup_expired()`

**Signature**: `pub async fn cleanup_expired(&self) -> Result<u64, RevokedTokenError>`

**Purpose**: Remove expired revocation records (background job)

**SQL Query**:
```sql
DELETE FROM revoked_tokens
WHERE expires_at < ?
```

**Returns**: Number of records deleted

**Scheduling**:
```rust
// Background cleanup task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        match repo.cleanup_expired().await {
            Ok(count) => info!("Cleaned up {} expired revocations", count),
            Err(e) => error!("Cleanup failed: {}", e),
        }
    }
});
```

---

### Method: `get_revocation_details()`

**Signature**: `pub async fn get_revocation_details(&self, token_jti: Uuid) -> Result<Option<RevokedTokenRecord>, RevokedTokenError>`

**Purpose**: Audit and debugging

**Returns**: Full revocation record with metadata

**Usage**:
```rust
// Audit log
if let Some(details) = repo.get_revocation_details(token_jti).await? {
    info!(
        "Token {} revoked by {:?} at {} for reason: {:?}",
        details.token_jti,
        details.revoked_by,
        details.revoked_at,
        details.revoked_reason
    );
}
```

---

### Method: `count_active_revocations()`

**Signature**: `pub async fn count_active_revocations(&self) -> Result<i64, RevokedTokenError>`

**Purpose**: Monitoring and metrics

**Returns**: Number of active (non-expired) revocations

**Metrics**:
```rust
// Prometheus metrics
let count = repo.count_active_revocations().await?;
metrics::gauge!("revoked_tokens_active", count as f64);
```

---

## Database Schema

```sql
CREATE TABLE revoked_tokens (
    id CHAR(36) PRIMARY KEY,
    token_jti CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    token_type VARCHAR(20) NOT NULL,
    revoked_at TIMESTAMP NOT NULL,
    revoked_by CHAR(36),
    revoked_reason VARCHAR(255),
    expires_at TIMESTAMP NOT NULL,
    
    INDEX idx_token_jti (token_jti),
    INDEX idx_user_id (user_id),
    INDEX idx_expires_at (expires_at),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
```

**Indexes**:
- `idx_token_jti` - Fast revocation checks
- `idx_user_id` - User-wide revocations
- `idx_expires_at` - Cleanup queries

---

## Token Revocation Patterns

### Pattern 1: Logout

```rust
pub async fn logout(
    token_jti: Uuid,
    user_id: Uuid,
    tenant_id: Uuid,
    expires_at: DateTime<Utc>,
    repo: &RevokedTokenRepository,
) -> Result<()> {
    repo.add_revoked_token(
        token_jti,
        user_id,
        tenant_id,
        TokenType::Access,
        Some(user_id),
        Some("User logout".to_string()),
        expires_at,
    ).await?;
    
    Ok(())
}
```

### Pattern 2: Password Change

```rust
pub async fn on_password_change(
    user_id: Uuid,
    tenant_id: Uuid,
    repo: &RevokedTokenRepository,
) -> Result<()> {
    // Revoke all tokens
    repo.revoke_all_user_tokens(
        user_id,
        tenant_id,
        Some(user_id),
        "Password changed - security measure".to_string(),
    ).await?;
    
    Ok(())
}
```

### Pattern 3: Security Incident

```rust
pub async fn handle_security_incident(
    user_id: Uuid,
    tenant_id: Uuid,
    admin_id: Uuid,
    repo: &RevokedTokenRepository,
) -> Result<()> {
    // Admin-initiated revocation
    repo.revoke_all_user_tokens(
        user_id,
        tenant_id,
        Some(admin_id),
        "Security incident - account compromised".to_string(),
    ).await?;
    
    // Alert security team
    alert_security_team(user_id, "All tokens revoked").await;
    
    Ok(())
}
```

---

## Trait Implementation: `RevokedTokenStore`

```rust
#[async_trait::async_trait]
impl RevokedTokenStore for RevokedTokenRepository {
    async fn add_to_blacklist(
        &self,
        jti: Uuid,
        user_id: Uuid,
        tenant_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AuthError> {
        self.add_revoked_token(
            jti,
            user_id,
            tenant_id,
            TokenType::Access,
            None,
            Some("Revoked via TokenEngine".to_string()),
            expires_at
        )
        .await
        .map(|_| ())
        .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, AuthError> {
        self.is_token_revoked(jti)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))
    }
}
```

---

## Performance Considerations

### 1. Index Optimization

```sql
-- Covering index for fast revocation checks
CREATE INDEX idx_token_jti_expires ON revoked_tokens(token_jti, expires_at);
```

### 2. Partitioning

```sql
-- Partition by month for easier cleanup
ALTER TABLE revoked_tokens
PARTITION BY RANGE (YEAR(expires_at) * 100 + MONTH(expires_at)) (
    PARTITION p202401 VALUES LESS THAN (202402),
    PARTITION p202402 VALUES LESS THAN (202403),
    -- ...
);
```

### 3. Caching

```rust
use moka::future::Cache;

pub struct CachedRevokedTokenRepository {
    repo: RevokedTokenRepository,
    cache: Cache<Uuid, bool>,
}

impl CachedRevokedTokenRepository {
    pub async fn is_token_revoked(&self, jti: Uuid) -> Result<bool> {
        // Check cache first
        if let Some(revoked) = self.cache.get(&jti).await {
            return Ok(revoked);
        }
        
        // Check database
        let revoked = self.repo.is_token_revoked(jti).await?;
        
        // Cache result (TTL: 5 minutes)
        self.cache.insert(jti, revoked).await;
        
        Ok(revoked)
    }
}
```

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_add_revoked_token(pool: MySqlPool) {
    let repo = RevokedTokenRepository::new(pool);
    
    let record = repo.add_revoked_token(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        TokenType::Access,
        None,
        Some("Test".to_string()),
        Utc::now() + Duration::hours(1),
    ).await.unwrap();
    
    assert!(repo.is_token_revoked(record.token_jti).await.unwrap());
}

#[sqlx::test]
async fn test_cleanup_expired(pool: MySqlPool) {
    let repo = RevokedTokenRepository::new(pool);
    
    // Add expired token
    repo.add_revoked_token(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        TokenType::Access,
        None,
        None,
        Utc::now() - Duration::hours(1), // Expired
    ).await.unwrap();
    
    let count = repo.cleanup_expired().await.unwrap();
    assert_eq!(count, 1);
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

- [auth-core/services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - RevokedTokenStore trait
- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [repositories/refresh_token_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/refresh_token_repository.md) - Refresh token management
- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - Token engine

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 287  
**Security Level**: CRITICAL
