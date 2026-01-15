# models/session.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/session.rs`  
**Crate**: `auth-core`  
**Module**: `models::session`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Tracks active user sessions and device fingerprints

## Purpose

Defines the `Session` model for tracking active user sessions, including device information, risk scores, and session lifecycle.

### Problem It Solves

- Tracks active authenticated sessions
- Enables session-based security (device fingerprinting, risk scoring)
- Supports session management (expiration, revocation)
- Provides audit trail for user activity

---

## Detailed Code Breakdown

### Struct: `Session`

**Purpose**: Represents an active user session

**Fields**:

| Field | Type | Description | Security Notes |
|-------|------|-------------|----------------|
| `id` | `Uuid` | Unique session identifier | Primary key |
| `user_id` | `Uuid` | Session owner | Foreign key to users |
| `tenant_id` | `Uuid` | Tenant context | Multi-tenancy isolation |
| `session_token` | `String` | Session token (hashed) | **CRITICAL**: Store hash, not plaintext |
| `device_fingerprint` | `Option<String>` | Device identifier hash | Security: Detect session hijacking |
| `user_agent` | `Option<String>` | Browser/client info | Audit trail |
| `ip_address` | `Option<String>` | Client IP address | Security: Geo-location checks |
| `risk_score` | `f32` | Session risk score (0.0-1.0) | Adaptive security |
| `last_activity` | `DateTime<Utc>` | Last request timestamp | Idle timeout detection |
| `expires_at` | `DateTime<Utc>` | Session expiration | Typically 24 hours |
| `created_at` | `DateTime<Utc>` | Session creation time | Audit trail |

**Derive Macros**:
- `Debug`: Debugging support
- `Clone`: Cheap cloning for Arc-wrapped instances
- `Serialize`, `Deserialize`: JSON serialization
- `sqlx::FromRow`: Automatic database row mapping

---

## Session Lifecycle

### 1. Session Creation

**Trigger**: Successful login

```rust
let session = Session {
    id: Uuid::new_v4(),
    user_id: user.id,
    tenant_id: tenant.id,
    session_token: sha256(&random_token),  // Hash before storage
    device_fingerprint: Some(compute_fingerprint(&request)),
    user_agent: Some(request.user_agent),
    ip_address: Some(request.ip),
    risk_score: 0.0,  // Initial risk
    last_activity: Utc::now(),
    expires_at: Utc::now() + Duration::hours(24),
    created_at: Utc::now(),
};
```

**Security**:
- Session token hashed (SHA-256)
- Device fingerprint computed from:
  - User-Agent
  - Screen resolution
  - Timezone
  - Language
  - Canvas fingerprint

### 2. Session Validation

**Every Request**:

```rust
// 1. Find session by token hash
let session = find_session_by_token(&token_hash).await?;

// 2. Check expiration
if session.expires_at < Utc::now() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// 3. Check idle timeout (30 minutes)
if session.last_activity + Duration::minutes(30) < Utc::now() {
    return Err(AuthError::TokenError { kind: TokenErrorKind::Expired });
}

// 4. Verify device fingerprint
if session.device_fingerprint != Some(current_fingerprint) {
    // Potential session hijacking
    increase_risk_score(&session.id, 0.5).await?;
    require_mfa = true;
}

// 5. Update last_activity
update_session_activity(&session.id).await?;
```

### 3. Session Refresh

**Extends Session**:

```rust
// Update expiration and last_activity
sqlx::query!(
    "UPDATE sessions SET last_activity = ?, expires_at = ? WHERE id = ?",
    Utc::now(),
    Utc::now() + Duration::hours(24),
    session.id
)
.execute(&pool)
.await?;
```

### 4. Session Termination

**Triggers**:
- User logout
- Session expiration
- Security event (password change, suspicious activity)

```rust
// Soft delete (for audit trail)
sqlx::query!(
    "UPDATE sessions SET revoked_at = ? WHERE id = ?",
    Utc::now(),
    session.id
)
.execute(&pool)
.await?;

// Or hard delete
sqlx::query!("DELETE FROM sessions WHERE id = ?", session.id)
    .execute(&pool)
    .await?;
```

---

## Device Fingerprinting

### Purpose

Detect session hijacking by binding session to device characteristics.

### Implementation

```rust
pub fn compute_device_fingerprint(request: &HttpRequest) -> String {
    let components = vec![
        request.user_agent.clone().unwrap_or_default(),
        request.headers.get("Accept-Language").unwrap_or_default(),
        request.headers.get("Accept-Encoding").unwrap_or_default(),
        // Canvas fingerprint (client-side)
        // Screen resolution (client-side)
        // Timezone (client-side)
    ];
    
    sha256(&components.join("|"))
}
```

### Security Considerations

**Pros**:
- Detects session theft across devices
- Increases attack difficulty

**Cons**:
- Browser updates may change fingerprint
- VPN/proxy changes may trigger false positives

**Mitigation**:
- Don't auto-revoke on fingerprint change
- Increase risk score and require MFA
- Allow user to verify new device

---

## Risk Scoring

### Risk Factors

1. **Device Fingerprint Mismatch**: +0.5
2. **IP Address Change**: +0.3
3. **Geo-location Change**: +0.4
4. **Unusual Activity Time**: +0.2
5. **High Request Rate**: +0.3

### Risk Actions

| Risk Score | Action |
|------------|--------|
| 0.0 - 0.3 | Normal operation |
| 0.3 - 0.5 | Log warning |
| 0.5 - 0.7 | Require email verification |
| 0.7 - 0.9 | Require MFA |
| 0.9 - 1.0 | Revoke session, require re-login |

---

## Session Management

### Concurrent Sessions

**Policy**: Allow multiple sessions per user

```rust
// Find all active sessions for user
let sessions = sqlx::query_as!(
    Session,
    "SELECT * FROM sessions WHERE user_id = ? AND expires_at > ? AND revoked_at IS NULL",
    user.id,
    Utc::now()
)
.fetch_all(&pool)
.await?;

// Limit to 5 concurrent sessions
if sessions.len() >= 5 {
    // Revoke oldest session
    revoke_session(&sessions[0].id).await?;
}
```

### Session Revocation

**Triggers**:
- User logout
- Password change (revoke all sessions)
- Account suspension (revoke all sessions)
- Security event (revoke specific session)

```rust
// Revoke all user sessions
pub async fn revoke_all_user_sessions(user_id: Uuid) -> Result<()> {
    sqlx::query!(
        "UPDATE sessions SET revoked_at = ? WHERE user_id = ? AND revoked_at IS NULL",
        Utc::now(),
        user_id
    )
    .execute(&pool)
    .await?;
    Ok(())
}
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
    ip_address VARCHAR(45),  -- IPv4 or IPv6
    risk_score FLOAT DEFAULT 0.0,
    last_activity TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at TIMESTAMP NULL,
    
    INDEX idx_user_id (user_id),
    INDEX idx_session_token (session_token),
    INDEX idx_expires_at (expires_at),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

---

## Security Considerations

### Session Token Security

1. **Hashed Storage**: Only SHA-256 hash stored
   - Prevents token theft from database breach
   - Original token never persisted

2. **Cryptographically Secure**: Generated with CSPRNG
   - 32 bytes of random data
   - Base64 encoded for transmission

3. **HttpOnly Cookies**: Token stored in HttpOnly cookie
   - Prevents XSS attacks
   - Automatic transmission with requests

### Session Hijacking Prevention

1. **Device Fingerprinting**: Binds session to device
2. **IP Validation**: Detects IP changes
3. **Risk Scoring**: Adaptive security
4. **Idle Timeout**: 30 minutes of inactivity
5. **Absolute Timeout**: 24 hours maximum

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `chrono` | Timestamp handling |
| `serde` | JSON serialization |
| `uuid` | Unique identifiers |
| `sqlx` | Database mapping |

### Internal Dependencies

- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [services/session_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/session_service.rs) - Session management

---

## Related Files

- [services/session_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/session_service.rs) - Session operations
- [repositories/session_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/session_repository.rs) - Session persistence
- [middleware/session.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware) - Session validation middleware

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 20  
**Security Level**: CRITICAL
