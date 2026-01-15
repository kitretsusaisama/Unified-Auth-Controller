# repositories/session_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/session_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::session_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ✅ **YES** - Session management and authentication state

## Purpose

Implements the `SessionStore` trait for MySQL/SQLite persistence, providing CRUD operations for user sessions with device tracking and risk scoring.

### Problem It Solves

- Persists active user sessions
- Tracks device fingerprints and user agents
- Enables session revocation and management
- Supports concurrent session limits

---

## Detailed Code Breakdown

### Struct: `SessionRepository`

**Purpose**: MySQL/SQLite implementation of `SessionStore` trait

**Fields**:
- `pool`: `Pool<MySql>` - Database connection pool

---

### Method: `SessionRepository::new()`

**Signature**: `pub fn new(pool: Pool<MySql>) -> Self`

**Purpose**: Constructor with connection pool

---

### Trait Implementation: `SessionStore for SessionRepository`

#### Method: `create()`

**Signature**: `async fn create(&self, session: Session) -> Result<Session, AuthError>`

**Purpose**: Create new session in database

**SQL Query**:
```sql
INSERT INTO sessions (
    id, user_id, tenant_id, session_token, device_fingerprint, 
    user_agent, ip_address, risk_score, last_activity, expires_at, created_at
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

**Bindings**:
1. `id` - Session UUID (as string)
2. `user_id` - User UUID (as string)
3. `tenant_id` - Tenant UUID (as string)
4. `session_token` - SHA-256 hash of session token
5. `device_fingerprint` - Device identifier (optional)
6. `user_agent` - Browser user agent (optional)
7. `ip_address` - Client IP address (optional)
8. `risk_score` - Risk assessment score (0.0-1.0)
9. `last_activity` - Current timestamp
10. `expires_at` - Session expiration time
11. `created_at` - Creation timestamp

**Returns**: Created session

**Security**:
- Session token stored as SHA-256 hash (never plaintext)
- Device fingerprint for session hijacking detection
- Risk score for adaptive security

---

#### Method: `get()`

**Signature**: `async fn get(&self, session_token: &str) -> Result<Option<Session>, AuthError>`

**Purpose**: Retrieve session by token hash

**SQL Query**:
```sql
SELECT * FROM sessions WHERE session_token = ?
```

**Returns**: `Option<Session>` (None if not found or expired)

**Usage**:
```rust
let session = session_repo.get(&token_hash).await?;
if let Some(s) = session {
    if s.expires_at > Utc::now() {
        // Valid session
    }
}
```

---

#### Method: `delete()`

**Signature**: `async fn delete(&self, session_token: &str) -> Result<(), AuthError>`

**Purpose**: Delete specific session (logout)

**SQL Query**:
```sql
DELETE FROM sessions WHERE session_token = ?
```

**Use Cases**:
- User logout
- Session revocation
- Security incident response

**Example**:
```rust
// Logout user
session_repo.delete(&session_token).await?;
```

---

#### Method: `delete_by_user()`

**Signature**: `async fn delete_by_user(&self, user_id: Uuid) -> Result<(), AuthError>`

**Purpose**: Delete all sessions for a user

**SQL Query**:
```sql
DELETE FROM sessions WHERE user_id = ?
```

**Use Cases**:
- "Logout from all devices"
- Account compromise response
- Password change (force re-authentication)
- Account suspension

**Example**:
```rust
// Force logout from all devices
session_repo.delete_by_user(user_id).await?;
```

---

## Database Schema

```sql
CREATE TABLE sessions (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    session_token VARCHAR(255) NOT NULL UNIQUE,  -- SHA-256 hash
    device_fingerprint VARCHAR(255),
    user_agent TEXT,
    ip_address VARCHAR(45),
    risk_score FLOAT DEFAULT 0.0,
    last_activity TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_user_id (user_id),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_session_token (session_token),
    INDEX idx_expires_at (expires_at),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
```

---

## Session Lifecycle

### 1. Session Creation

```rust
// After successful login
let session = Session {
    id: Uuid::new_v4(),
    user_id,
    tenant_id,
    session_token: hash_token(&raw_token),  // SHA-256
    device_fingerprint: Some(fingerprint),
    user_agent: Some(user_agent),
    ip_address: Some(ip),
    risk_score: 0.0,
    last_activity: Utc::now(),
    expires_at: Utc::now() + Duration::hours(24),
    created_at: Utc::now(),
};

session_repo.create(session).await?;
```

### 2. Session Validation

```rust
// On each request
let session = session_repo.get(&token_hash).await?
    .ok_or(AuthError::Unauthorized { message: "Invalid session".to_string() })?;

// Check expiration
if session.expires_at < Utc::now() {
    session_repo.delete(&token_hash).await?;
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// Check risk score
if session.risk_score > 0.8 {
    // Require re-authentication or MFA
}
```

### 3. Session Refresh

```rust
// Update last activity
sqlx::query!(
    "UPDATE sessions SET last_activity = ? WHERE session_token = ?",
    Utc::now(),
    token_hash
).execute(&pool).await?;
```

### 4. Session Termination

```rust
// Logout
session_repo.delete(&token_hash).await?;

// Logout all devices
session_repo.delete_by_user(user_id).await?;
```

---

## Security Features

### 1. Token Hashing

**Implementation**:
```rust
use sha2::{Sha256, Digest};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Benefits**:
- Database compromise doesn't expose session tokens
- Tokens can't be used if database is leaked

### 2. Device Fingerprinting

**Purpose**: Detect session hijacking

**Implementation**:
```rust
fn generate_fingerprint(headers: &HeaderMap) -> String {
    let user_agent = headers.get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    let accept_language = headers.get("Accept-Language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    // Hash combination
    hash(&format!("{}{}", user_agent, accept_language))
}
```

**Validation**:
```rust
if session.device_fingerprint != Some(current_fingerprint) {
    // Possible session hijacking
    session_repo.delete(&token_hash).await?;
    return Err(AuthError::Unauthorized { 
        message: "Device mismatch".to_string() 
    });
}
```

### 3. Risk Scoring

**Factors**:
- IP address changes
- Unusual access patterns
- Geographic anomalies
- Time-based patterns

**Response**:
```rust
if session.risk_score > 0.5 {
    // Require MFA
} else if session.risk_score > 0.8 {
    // Force re-authentication
    session_repo.delete(&token_hash).await?;
}
```

---

## Concurrent Session Management

### Limit Sessions Per User

```rust
pub async fn enforce_session_limit(
    pool: &MySqlPool,
    user_id: Uuid,
    max_sessions: usize,
) -> Result<()> {
    // Get all sessions for user
    let sessions = sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE user_id = ? ORDER BY created_at DESC",
        user_id.to_string()
    )
    .fetch_all(pool)
    .await?;
    
    // Delete oldest sessions if over limit
    if sessions.len() > max_sessions {
        for session in sessions.iter().skip(max_sessions) {
            sqlx::query!(
                "DELETE FROM sessions WHERE id = ?",
                session.id.to_string()
            )
            .execute(pool)
            .await?;
        }
    }
    
    Ok(())
}
```

---

## Cleanup Operations

### Delete Expired Sessions

```rust
pub async fn cleanup_expired_sessions(pool: &MySqlPool) -> Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM sessions WHERE expires_at < ?",
        Utc::now()
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected())
}
```

**Scheduling**: Run as cron job or background task

```rust
// Background task
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        if let Err(e) = cleanup_expired_sessions(&pool).await {
            error!("Failed to cleanup sessions: {}", e);
        }
    }
});
```

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_create_session(pool: MySqlPool) {
    let repo = SessionRepository::new(pool);
    
    let session = Session {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        session_token: "hashed_token".to_string(),
        device_fingerprint: Some("fingerprint".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        ip_address: Some("192.168.1.1".to_string()),
        risk_score: 0.0,
        last_activity: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        created_at: Utc::now(),
    };
    
    let created = repo.create(session.clone()).await.unwrap();
    assert_eq!(created.id, session.id);
}

#[sqlx::test]
async fn test_delete_by_user(pool: MySqlPool) {
    let repo = SessionRepository::new(pool);
    let user_id = Uuid::new_v4();
    
    // Create multiple sessions
    for _ in 0..3 {
        let session = create_test_session(user_id);
        repo.create(session).await.unwrap();
    }
    
    // Delete all
    repo.delete_by_user(user_id).await.unwrap();
    
    // Verify deleted
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sessions WHERE user_id = ?",
        user_id.to_string()
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(count, 0);
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database operations |
| `uuid` | Session identifiers |
| `anyhow` | Error handling |
| `async-trait` | Async trait support |

### Internal Dependencies

- [auth-core/models/session.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/session.rs) - Session entity
- [auth-core/services/session_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - SessionStore trait
- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [models/session.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/session.md) - Session model
- [services/session_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Session service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 75  
**Security Level**: CRITICAL
