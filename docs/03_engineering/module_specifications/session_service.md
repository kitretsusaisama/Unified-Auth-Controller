---
title: Session Service Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Module Specification
crate: auth-core
---

# Session Service Specification

> [!NOTE]
> **Module**: `auth-core::services::session_service`  
> **Responsibility**: Session lifecycle and security (fingerprinting, Sudo Mode)

---

## 1. Overview

The **Session Service** manages user sessions, including creation, validation, fingerprinting, and security features like Sudo Mode for critical actions.

---

## 2. Public API

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: CreateSessionRequest) 
        -> Result<Session, AuthError>;
    
    async fn find_by_id(&self, id: Uuid) 
        -> Result<Option<Session>, AuthError>;
    
    async fn validate(&self, id: Uuid, fingerprint: &SessionFingerprint) 
        -> Result<bool, AuthError>;
    
    async fn revoke(&self, id: Uuid) 
        -> Result<(), AuthError>;
    
    async fn revoke_all_for_user(&self, user_id: Uuid) 
        -> Result<(), AuthError>;
}
```

---

## 3. Models

### 3.1 Session

```rust
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub fingerprint: SessionFingerprint,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}
```

### 3.2 Session Fingerprint

```rust
pub struct SessionFingerprint {
    pub ip_address: String,
    pub user_agent: String,
    pub device_id: Option<String>,
}
```

---

## 4. Operations

### 4.1 Create Session

**Method**: `create(request: CreateSessionRequest) -> Result<Session>`

**Flow**:
1. Generate session ID
2. Capture fingerprint (IP, User-Agent, device)
3. Set expiration (24 hours)
4. Store in database
5. Return session

---

### 4.2 Validate Session

**Method**: `validate(id: Uuid, fingerprint: &SessionFingerprint) -> Result<bool>`

**Flow**:
1. Fetch session by ID
2. Check expiration
3. Compare fingerprint (IP, User-Agent)
4. Update last_activity
5. Return valid/invalid

**Security**:
- Fingerprint mismatch = session hijacking attempt
- Automatic revocation on suspicious activity

---

### 4.3 Sudo Mode (Planned)

**Purpose**: Require re-authentication for critical actions.

**Flow**:
1. User performs critical action (delete account, change email)
2. Check if in Sudo Mode (last auth < 5 min)
3. If not, require password re-entry
4. Enter Sudo Mode for 5 minutes

---

## 5. Security Considerations

- **Fingerprinting**: Detect session hijacking
- **Expiration**: Automatic cleanup of old sessions
- **Concurrent Limits**: Max 5 sessions per user
- **Revocation**: Immediate effect (logout all devices)

---

**Document Status**: Active  
**Owner**: Engineering Team
