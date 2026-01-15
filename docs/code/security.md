# Security & Trust Boundaries

This document provides comprehensive security documentation for the Enterprise SSO Platform, covering authentication boundaries, authorization models, cryptographic responsibilities, and attack surface minimization.

## Table of Contents

1. [Authentication Boundaries](#authentication-boundaries)
2. [Authorization Model](#authorization-model)
3. [Token Lifecycle](#token-lifecycle)
4. [Cryptographic Responsibilities](#cryptographic-responsibilities)
5. [Secrets Handling](#secrets-handling)
6. [Attack Surface Minimization](#attack-surface-minimization)
7. [Audit Trail & Compliance](#audit-trail--compliance)

---

## Authentication Boundaries

### Trust Zones

The platform defines clear trust boundaries:

```
┌─────────────────────────────────────────────────────────┐
│                    Public Zone                          │
│  - Login endpoints                                      │
│  - Registration endpoints                               │
│  - Password reset                                       │
│  - Health checks                                        │
└─────────────────────────────────────────────────────────┘
                        ↓
                 [Authentication]
                        ↓
┌─────────────────────────────────────────────────────────┐
│                 Authenticated Zone                      │
│  - User profile access                                  │
│  - Token refresh                                        │
│  - Basic API operations                                 │
└─────────────────────────────────────────────────────────┘
                        ↓
                  [Authorization]
                        ↓
┌─────────────────────────────────────────────────────────┐
│                  Authorized Zone                        │
│  - Administrative operations                            │
│  - Sensitive data access                                │
│  - Configuration changes                                │
└─────────────────────────────────────────────────────────┘
```

### Middleware Chain

**File**: [auth-api/src/router.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/router.rs)

Authentication enforcement order:

1. **Request ID Generation** (`middleware/request_id.rs`)
   - Assigns UUID to every request
   - Enables distributed tracing

2. **Security Headers** (`middleware/security_headers.rs`)
   - Content-Security-Policy
   - X-Frame-Options: DENY
   - X-Content-Type-Options: nosniff
   - Strict-Transport-Security

3. **Rate Limiting** (`middleware/rate_limit.rs`)
   - 5 requests/minute per IP address
   - Token bucket algorithm
   - Prevents brute-force attacks

4. **JWT Validation** (for protected routes)
   - Extract Bearer token from Authorization header
   - Verify signature (RS256)
   - Check expiration
   - Validate claims (issuer, audience, tenant)

5. **Permission Checking** (for admin routes)
   - Load user permissions from cache/database
   - Check required permissions for endpoint
   - Enforce RBAC/ABAC rules

### Authentication Methods

#### 1. Password-Based Authentication
**Security Level**: Medium  
**Implementation**: [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs)

**Security Measures**:
- Argon2id password hashing (OWASP recommended)
- Per-user salts (generated with CSPRNG)
- Minimum 8 characters, configurable complexity
- Common password blacklist
- Account lockout after 5 failed attempts (30-minute lockout)
- Constant-time password comparison

#### 2. Multi-Factor Authentication (MFA)
**Security Level**: High  
**Implementation**: [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs)

**Methods**:
- **TOTP** (Time-Based One-Time Password, RFC 6238)
  - 6-digit codes
  - 30-second time window
  - ±1 window tolerance for clock skew
- **Backup Codes**
  - 10 single-use codes
  - Hashed before storage
  - Regenerated after use

**Adaptive MFA**:
- Triggered by risk score > 0.7
- Factors: IP address, device fingerprint, impossible travel

#### 3. WebAuthn (Passwordless)
**Security Level**: Very High  
**Implementation**: [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs)

**Security Measures**:
- Public key cryptography
- Hardware security key support (YubiKey, etc.)
- Biometric authentication (fingerprint, Face ID)
- Phishing-resistant (origin-bound credentials)

#### 4. OAuth/OIDC (Federated)
**Security Level**: High (depends on provider)  
**Implementation**: [protocols/oidc.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-protocols/src/oidc.rs)

**Security Measures**:
- PKCE (Proof Key for Code Exchange)
- CSRF protection with state parameter
- ID token signature verification
- Nonce validation
- Issuer validation

---

## Authorization Model

### Role-Based Access Control (RBAC)

**Implementation**: [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs)

#### Role Hierarchy

```
SuperAdmin
  ├── TenantAdmin
  │     ├── UserManager
  │     └── AuditorRole
  └── SystemOperator
```

#### Permission Model

**Structure**:
```rust
pub struct Permission {
    pub id: Uuid,
    pub resource: String,      // e.g., "users", "roles", "audit_logs"
    pub action: String,        // e.g., "read", "write", "delete"
    pub scope: PermissionScope, // Global, Tenant, Organization
}
```

**Examples**:
- `users:read:tenant` - Read users within tenant
- `users:write:global` - Create/update users globally
- `audit_logs:read:organization` - Read audit logs for organization
- `roles:delete:tenant` - Delete roles within tenant

#### Permission Checking

**File**: [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs)

```rust
pub async fn check_permission(
    &self,
    user_id: Uuid,
    resource: &str,
    action: &str,
    scope: PermissionScope,
) -> Result<bool, AuthError>
```

**Algorithm**:
1. Load user's roles from cache/database
2. For each role, load associated permissions
3. Check if any permission matches (resource, action, scope)
4. Cache result for 5 minutes

**Performance**: 
- Cache hit: <5ms
- Cache miss: <20ms (database query)

### Attribute-Based Access Control (ABAC)

**Use Cases**:
- Time-based access (business hours only)
- IP-based access (internal network only)
- Data ownership (user can only modify their own data)

**Example**:
```rust
if user.department == "Finance" && 
   current_time.hour() >= 9 && 
   current_time.hour() <= 17 {
    allow_access();
}
```

---

## Token Lifecycle

### Access Token

**Type**: JWT (JSON Web Token)  
**Algorithm**: RS256 (RSA with SHA-256)  
**Expiration**: 15 minutes  
**Storage**: Client-side (memory, not localStorage)

#### Token Structure

```json
{
  "header": {
    "alg": "RS256",
    "typ": "JWT",
    "kid": "key-2024-01"
  },
  "payload": {
    "sub": "user-uuid",
    "iss": "auth-service",
    "aud": "auth-service",
    "exp": 1705234567,
    "iat": 1705233667,
    "nbf": 1705233667,
    "jti": "token-uuid",
    "tenant_id": "tenant-uuid",
    "permissions": ["users:read:tenant"],
    "roles": ["UserManager"]
  },
  "signature": "..."
}
```

#### Validation Process

**File**: [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs)

1. **Signature Verification**
   - Fetch public key by `kid` (key ID)
   - Verify RSA signature
   - Reject if signature invalid

2. **Expiration Check**
   - Compare `exp` with current time
   - Reject if expired

3. **Issuer Validation**
   - Verify `iss` matches expected issuer
   - Reject if issuer mismatch

4. **Audience Validation**
   - Verify `aud` matches expected audience
   - Reject if audience mismatch

5. **Not Before Check**
   - Compare `nbf` with current time
   - Reject if token used before valid time

6. **Revocation Check**
   - Check `jti` against revocation list
   - Reject if revoked

### Refresh Token

**Type**: Opaque token (random bytes)  
**Algorithm**: SHA-256 hash of random 32-byte value  
**Expiration**: 7 days  
**Storage**: Database (hashed), client-side (HttpOnly cookie)

#### Security Features

1. **Single-Use with Rotation**
   - Each refresh generates new access + refresh tokens
   - Old refresh token immediately revoked
   - Prevents token replay attacks

2. **Hashed Storage**
   - Only SHA-256 hash stored in database
   - Original token never persisted
   - Prevents token theft from database breach

3. **Revocation List**
   - Used tokens added to revocation list
   - TTL matches token expiration
   - Prevents reuse of old tokens

#### Refresh Flow Security

**File**: [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs)

```rust
pub async fn refresh_access_token(&self, refresh_token: &str) 
    -> Result<(AccessToken, RefreshToken), AuthError>
{
    // 1. Hash the provided token
    let token_hash = sha256(refresh_token);
    
    // 2. Find token in database
    let stored_token = self.refresh_repo.find_by_hash(&token_hash).await?
        .ok_or(AuthError::InvalidToken)?;
    
    // 3. Check expiration
    if stored_token.expires_at < Utc::now() {
        return Err(AuthError::TokenExpired);
    }
    
    // 4. Check revocation
    if self.revoked_repo.is_revoked(&token_hash).await? {
        return Err(AuthError::TokenRevoked);
    }
    
    // 5. Generate new tokens
    let new_access = self.issue_access_token(claims).await?;
    let new_refresh = self.issue_refresh_token(user_id, tenant_id).await?;
    
    // 6. Revoke old token
    self.refresh_repo.delete(&stored_token.id).await?;
    self.revoked_repo.add(&token_hash, stored_token.expires_at).await?;
    
    Ok((new_access, new_refresh))
}
```

---

## Cryptographic Responsibilities

### Password Hashing

**Algorithm**: Argon2id  
**Implementation**: [auth-crypto/src/password.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-crypto/src/password.rs)

**Parameters**:
```rust
Argon2::default() // Uses recommended defaults:
// - Memory: 19 MiB
// - Iterations: 2
// - Parallelism: 1
// - Salt: 16 bytes (CSPRNG)
// - Output: 32 bytes
```

**Why Argon2id**:
- Winner of Password Hashing Competition (2015)
- Resistant to GPU/ASIC attacks
- Resistant to side-channel attacks
- OWASP recommended
- Configurable memory hardness

**Security**:
- Per-user salts (never reused)
- Constant-time verification
- Automatic parameter tuning

### JWT Signing

**Algorithm**: RS256 (RSA with SHA-256)  
**Implementation**: [auth-crypto/src/jwt.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-crypto/src/jwt.rs)

**Key Management**:
```rust
pub struct KeyPair {
    pub kid: String,           // Key ID (e.g., "key-2024-01")
    pub private_key: RsaPrivateKey, // 2048-bit RSA private key
    pub public_key: RsaPublicKey,   // Corresponding public key
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
```

**Key Rotation**:
- New key generated monthly
- Old keys retained for 30 days (for token validation)
- Automatic rotation on compromise detection
- Multiple active keys supported (identified by `kid`)

**Why RS256 over HS256**:
- Asymmetric: Public key can be shared for validation
- Private key never leaves auth service
- Prevents token forgery by API consumers
- Supports distributed validation

### TLS Configuration

**Version**: TLS 1.3 only  
**Cipher Suites**: Modern, secure ciphers only

**Rejected**:
- TLS 1.0, 1.1, 1.2 (legacy protocols)
- Weak ciphers (RC4, DES, 3DES)
- Export-grade ciphers

**Certificate Management**:
- Automatic renewal (Let's Encrypt)
- Certificate pinning (optional, for high-security)
- OCSP stapling for revocation checking

---

## Secrets Handling

### Environment Variables

**File**: `.env` (gitignored)

**Critical Secrets**:
```bash
DATABASE_URL=mysql://user:password@localhost/auth_platform
JWT_PRIVATE_KEY_PATH=/secrets/jwt_private_key.pem
JWT_PUBLIC_KEY_PATH=/secrets/jwt_public_key.pem
ENCRYPTION_KEY=base64-encoded-32-byte-key
REDIS_URL=redis://:password@localhost:6379
```

**Security Measures**:
1. **Never Committed**: `.env` in `.gitignore`
2. **Encrypted at Rest**: Secrets encrypted in production (Vault, AWS Secrets Manager)
3. **Least Privilege**: Each service has minimal required secrets
4. **Rotation**: Regular secret rotation (90 days)

### Secrecy Crate

**Implementation**: Throughout codebase

```rust
use secrecy::{Secret, ExposeSecret};

pub struct Config {
    pub jwt_secret: Secret<String>,
    pub database_password: Secret<String>,
}

// Secrets never logged
impl Debug for Config {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Config")
            .field("jwt_secret", &"[REDACTED]")
            .field("database_password", &"[REDACTED]")
            .finish()
    }
}

// Explicit exposure required
let password = config.database_password.expose_secret();
```

**Benefits**:
- Prevents accidental logging
- Explicit secret exposure
- Memory zeroization on drop
- Type-safe secret handling

### No Secrets in Logs

**Implementation**: [auth-telemetry/src/tracing.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-telemetry/src/tracing.rs)

**Filtered Fields**:
- `password`
- `token`
- `secret`
- `api_key`
- `authorization`

**Example**:
```rust
tracing::info!(
    user_id = %user.id,
    email = %user.email,
    password = "[REDACTED]", // Never logged
    "User login attempt"
);
```

---

## Attack Surface Minimization

### SQL Injection Prevention

**Implementation**: SQLx compile-time checked queries

**Safe**:
```rust
sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = ? AND tenant_id = ?",
    email,
    tenant_id
)
.fetch_one(&pool)
.await?
```

**Unsafe** (never used):
```rust
// NEVER DO THIS
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

**Protection**:
- All queries parameterized
- Compile-time type checking
- No dynamic SQL construction

### Cross-Site Scripting (XSS) Prevention

**Implementation**: [middleware/security_headers.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/security_headers.rs)

**Content-Security-Policy**:
```
Content-Security-Policy: 
  default-src 'self'; 
  script-src 'self'; 
  style-src 'self' 'unsafe-inline'; 
  img-src 'self' data: https:; 
  font-src 'self'; 
  connect-src 'self'; 
  frame-ancestors 'none';
```

**Additional Headers**:
- `X-Content-Type-Options: nosniff` - Prevents MIME sniffing
- `X-Frame-Options: DENY` - Prevents clickjacking
- `X-XSS-Protection: 1; mode=block` - Legacy XSS protection

### Cross-Site Request Forgery (CSRF) Prevention

**Implementation**: Token-based CSRF protection

**Mechanism**:
1. Generate CSRF token on session creation
2. Include token in forms/requests
3. Validate token on state-changing operations

**Exemptions**:
- API endpoints using JWT (stateless)
- Read-only operations (GET requests)

### Rate Limiting

**Implementation**: [middleware/rate_limit.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/rate_limit.rs)

**Algorithm**: Token Bucket

**Limits**:
- **Login**: 5 attempts/minute per IP
- **Registration**: 3 attempts/hour per IP
- **Password Reset**: 3 attempts/hour per email
- **API**: 100 requests/minute per user

**Storage**: DashMap (in-memory) or Redis (distributed)

**Bypass**: Whitelisted IPs (internal services)

### Account Lockout

**Implementation**: [repositories/user_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/user_repository.rs)

**Trigger**: 5 failed login attempts within 15 minutes

**Lockout Duration**: 30 minutes

**Reset**: Automatic after lockout period, or manual by admin

**Notification**: Email sent to user on lockout

---

## Audit Trail & Compliance

### Immutable Audit Logs

**Implementation**: [auth-audit/src/storage.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-audit/src/storage.rs)

**Storage**: Append-only database table

**Schema**:
```sql
CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    user_id UUID,
    tenant_id UUID NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    resource VARCHAR(100),
    action VARCHAR(50),
    result VARCHAR(20), -- SUCCESS, FAILURE, DENIED
    metadata JSONB,
    timestamp TIMESTAMP NOT NULL,
    signature VARCHAR(128) NOT NULL -- HMAC for tamper detection
);
```

**Tamper Detection**:
```rust
let signature = hmac_sha256(
    &secret_key,
    format!("{}{}{}{}", event.id, event.timestamp, event.event_type, event.user_id)
);
```

### Compliance Features

#### SOC 2 Type II
- Comprehensive audit logging
- Access controls (RBAC)
- Encryption at rest and in transit
- Regular security assessments

#### HIPAA
- PHI encryption
- Audit trail for all PHI access
- Access controls
- Data retention policies

#### PCI-DSS
- No storage of full credit card numbers
- Encryption of cardholder data
- Access logging
- Regular security testing

#### GDPR
- Right to access (data export)
- Right to erasure (soft delete)
- Data portability
- Consent management

### Audit Event Types

**Authentication**:
- `LOGIN_SUCCESS`
- `LOGIN_FAILURE`
- `LOGOUT`
- `MFA_ENABLED`
- `MFA_VERIFIED`
- `PASSWORD_CHANGED`

**Authorization**:
- `PERMISSION_GRANTED`
- `PERMISSION_DENIED`
- `ROLE_ASSIGNED`
- `ROLE_REMOVED`

**Administrative**:
- `USER_CREATED`
- `USER_SUSPENDED`
- `USER_DELETED`
- `CONFIG_CHANGED`

**Data Access**:
- `PROFILE_VIEWED`
- `SENSITIVE_DATA_ACCESSED`
- `AUDIT_LOG_EXPORTED`

---

## Security Best Practices

### Defense in Depth

Multiple layers of security:
1. Network (TLS, firewall)
2. Application (authentication, authorization)
3. Data (encryption, hashing)
4. Monitoring (audit logs, alerts)

### Principle of Least Privilege

- Users have minimum required permissions
- Services have minimum required secrets
- Database users have minimum required grants

### Fail Securely

- Default deny for authorization
- Explicit error handling
- No information leakage in errors

### Security by Default

- Secure defaults in configuration
- Opt-in for insecure features
- Conservative security settings

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Security Level**: Enterprise-Grade  
**Compliance**: SOC 2, HIPAA, PCI-DSS, GDPR
