# services/session_service.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/session_service.rs`  
**Crate**: `auth-core`  
**Module**: `services::session_service`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - Session management and risk assessment

## Purpose

Orchestrates session lifecycle management with integrated risk assessment, providing secure session creation, validation, and revocation with adaptive security policies.

### Problem It Solves

- Creates sessions with risk-based security
- Validates session tokens and expiration
- Revokes individual or all user sessions
- Blocks high-risk login attempts
- Integrates device fingerprinting

---

## Detailed Code Breakdown

### Trait: `SessionStore`

**Purpose**: Persistence abstraction for sessions

**Methods**:

| Method | Purpose |
|--------|---------|
| `create()` | Persist new session |
| `get()` | Retrieve session by token |
| `delete()` | Delete single session |
| `delete_by_user()` | Delete all user sessions |

---

### Struct: `SessionService`

**Purpose**: Session business logic with risk assessment

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `store` | `Arc<dyn SessionStore>` | Session persistence |
| `risk_engine` | `Arc<dyn RiskAssessor>` | Risk assessment engine |

---

### Method: `SessionService::new()`

**Signature**: `pub fn new(store: Arc<dyn SessionStore>, risk_engine: Arc<dyn RiskAssessor>) -> Self`

**Purpose**: Constructor with dependencies

**Example**:
```rust
let session_service = SessionService::new(
    Arc::new(SessionRepository::new(pool.clone())),
    Arc::new(RiskEngine::new()),
);
```

---

### Method: `create_session()`

**Signature**: `pub async fn create_session(&self, user: User, risk_context: RiskContext) -> Result<Session, AuthError>`

**Purpose**: Create session with risk assessment

**Process**:

#### 1. Assess Risk
```rust
let risk_assessment = self.risk_engine.assess_risk(risk_context.clone()).await?;
```

**Risk Factors**:
- IP address reputation
- Device fingerprint
- Geographic location
- Time of day
- Velocity (login frequency)

#### 2. Apply Security Policy
```rust
if risk_assessment.score >= 0.9 {
    return Err(AuthError::AuthorizationDenied { 
        permission: "login".to_string(), 
        resource: "session".to_string() 
    });
}
```

**Risk Levels**:
- `0.0-0.3`: Low - Allow
- `0.3-0.7`: Medium - Allow with monitoring
- `0.7-0.9`: High - Require MFA
- `0.9-1.0`: Critical - Block

#### 3. Create Session
```rust
let session = Session {
    id: Uuid::new_v4(),
    user_id: user.id,
    tenant_id: /* from context */,
    session_token: Uuid::new_v4().to_string(), // Use crypto-random in production
    device_fingerprint: risk_context.device_fingerprint,
    user_agent: risk_context.user_agent,
    ip_address: risk_context.ip_address,
    risk_score: risk_assessment.score,
    last_activity: Utc::now(),
    expires_at: Utc::now() + Duration::minutes(60),
    created_at: Utc::now(),
};

self.store.create(session).await
```

**Returns**: Created session

---

### Method: `validate_session()`

**Signature**: `pub async fn validate_session(&self, token: &str) -> Result<Session, AuthError>`

**Purpose**: Validate session token

**Process**:

#### 1. Retrieve Session
```rust
let session = self.store.get(token).await?
    .ok_or(AuthError::AuthenticationFailed { 
        reason: "Session invalid".to_string() 
    })?;
```

#### 2. Check Expiration
```rust
if session.expires_at < Utc::now() {
    self.store.delete(token).await?;
    return Err(AuthError::AuthenticationFailed { 
        reason: "Session expired".to_string() 
    });
}
```

#### 3. Return Valid Session
```rust
Ok(session)
```

**Usage**:
```rust
// In middleware
let session = session_service.validate_session(&token).await?;
req.extensions_mut().insert(session);
```

---

### Method: `revoke_session()`

**Signature**: `pub async fn revoke_session(&self, token: &str) -> Result<(), AuthError>`

**Purpose**: Revoke single session (logout)

**Example**:
```rust
// User logout
session_service.revoke_session(&session_token).await?;
```

---

### Method: `revoke_user_sessions()`

**Signature**: `pub async fn revoke_user_sessions(&self, user_id: Uuid) -> Result<(), AuthError>`

**Purpose**: Revoke all user sessions

**Use Cases**:
- "Logout from all devices"
- Password change
- Account compromise
- Security incident

**Example**:
```rust
// After password change
session_service.revoke_user_sessions(user_id).await?;
```

---

## Risk Assessment Integration

### Risk Context

```rust
pub struct RiskContext {
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub location: Option<GeoLocation>,
    pub timestamp: DateTime<Utc>,
}
```

### Risk Assessment

```rust
pub struct RiskAssessment {
    pub score: f64,           // 0.0 to 1.0
    pub level: RiskLevel,     // Low, Medium, High, Critical
    pub factors: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
}
```

### Example Flow

```rust
// Login attempt
let risk_context = RiskContext {
    user_id: Some(user.id),
    ip_address: Some("192.168.1.1".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    device_fingerprint: Some(fingerprint),
    location: Some(geo_location),
    timestamp: Utc::now(),
};

match session_service.create_session(user, risk_context).await {
    Ok(session) => {
        // Session created
        if session.risk_score > 0.7 {
            // High risk - require MFA
            require_mfa(&user).await?;
        }
        Ok(session)
    }
    Err(AuthError::AuthorizationDenied { .. }) => {
        // Critical risk - blocked
        alert_security_team(user.id, "High-risk login blocked").await;
        Err(AuthError::AuthorizationDenied { .. })
    }
    Err(e) => Err(e),
}
```

---

## Session Lifecycle

### 1. Creation (Login)
```rust
let session = session_service.create_session(user, risk_context).await?;
// Return session token to client
```

### 2. Validation (Each Request)
```rust
let session = session_service.validate_session(&token).await?;
// Proceed with request
```

### 3. Activity Update
```rust
sqlx::query!(
    "UPDATE sessions SET last_activity = ? WHERE session_token = ?",
    Utc::now(),
    token
).execute(&pool).await?;
```

### 4. Revocation (Logout)
```rust
session_service.revoke_session(&token).await?;
```

---

## Security Patterns

### Pattern 1: Sliding Expiration

```rust
pub async fn refresh_session_expiry(
    session_service: &SessionService,
    token: &str,
) -> Result<()> {
    let session = session_service.validate_session(token).await?;
    
    // Extend expiration
    sqlx::query!(
        "UPDATE sessions SET expires_at = ?, last_activity = ? WHERE session_token = ?",
        Utc::now() + Duration::minutes(60),
        Utc::now(),
        token
    ).execute(&pool).await?;
    
    Ok(())
}
```

### Pattern 2: Concurrent Session Limits

```rust
pub async fn enforce_session_limit(
    store: &dyn SessionStore,
    user_id: Uuid,
    max_sessions: usize,
) -> Result<()> {
    let sessions = get_user_sessions(user_id).await?;
    
    if sessions.len() >= max_sessions {
        // Delete oldest session
        store.delete(&sessions.last().unwrap().session_token).await?;
    }
    
    Ok(())
}
```

### Pattern 3: Device Binding

```rust
pub async fn validate_device(
    session: &Session,
    current_fingerprint: &str,
) -> Result<()> {
    if let Some(stored_fingerprint) = &session.device_fingerprint {
        if stored_fingerprint != current_fingerprint {
            // Device mismatch - possible session hijacking
            return Err(AuthError::AuthenticationFailed {
                reason: "Device mismatch".to_string()
            });
        }
    }
    Ok(())
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_create_session_low_risk() {
    let store = Arc::new(MockSessionStore::new());
    let risk_engine = Arc::new(MockRiskEngine::new(0.2)); // Low risk
    let service = SessionService::new(store, risk_engine);
    
    let user = create_test_user();
    let risk_context = create_test_risk_context();
    
    let session = service.create_session(user, risk_context).await.unwrap();
    assert!(session.risk_score < 0.3);
}

#[tokio::test]
async fn test_create_session_critical_risk_blocked() {
    let store = Arc::new(MockSessionStore::new());
    let risk_engine = Arc::new(MockRiskEngine::new(0.95)); // Critical risk
    let service = SessionService::new(store, risk_engine);
    
    let user = create_test_user();
    let risk_context = create_test_risk_context();
    
    let result = service.create_session(user, risk_context).await;
    assert!(matches!(result, Err(AuthError::AuthorizationDenied { .. })));
}

#[tokio::test]
async fn test_validate_expired_session() {
    let store = Arc::new(MockSessionStore::new());
    let risk_engine = Arc::new(MockRiskEngine::new(0.1));
    let service = SessionService::new(store.clone(), risk_engine);
    
    // Create expired session
    let mut session = create_test_session();
    session.expires_at = Utc::now() - Duration::minutes(1);
    store.insert(session.clone()).await;
    
    let result = service.validate_session(&session.session_token).await;
    assert!(matches!(result, Err(AuthError::AuthenticationFailed { .. })));
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `uuid` | Session identifiers |
| `chrono` | Timestamps |
| `async-trait` | Async trait support |

### Internal Dependencies

- [models/session.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/session.md) - Session model
- [services/risk_assessment.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/risk_assessment.rs) - Risk engine
- [error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [models/session.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/session.md) - Session model
- [repositories/session_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/session_repository.md) - Session persistence
- [services/risk_assessment.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/risk_assessment.rs) - Risk assessment

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 80  
**Security Level**: CRITICAL
