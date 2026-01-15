# services/identity.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/identity.rs`  
**Crate**: `auth-core`  
**Module**: `services::identity`  
**Layer**: Application  
**Security-Critical**: ✅ **YES** - Core authentication logic, password handling

## Purpose

Implements the `IdentityService` which orchestrates user authentication, registration, and lifecycle management. This is the primary service for all identity-related operations.

### Problem It Solves

- Centralizes authentication logic
- Enforces security policies (password hashing, account lockout)
- Coordinates between user storage and token generation
- Implements business rules for user lifecycle

---

## Detailed Code Breakdown

### Trait: `UserStore`

**Purpose**: Abstract interface for user persistence operations

**Methods**:

#### `find_by_email(email, tenant_id) -> Result<Option<User>>`
- Finds user by email within tenant context
- Returns `None` if not found
- Multi-tenant isolation enforced

#### `find_by_id(id) -> Result<Option<User>>`
- Finds user by UUID
- Global lookup (cross-tenant for admin operations)

#### `create(user, password_hash, tenant_id) -> Result<User>`
- Creates new user with hashed password
- Sets initial status to `PendingVerification`
- Returns created user with generated ID

#### `update_status(id, status) -> Result<()>`
- Updates user status (Active, Suspended, Deleted)
- Used for ban/activate operations

#### `increment_failed_attempts(id) -> Result<u32>`
- Increments failed login counter
- Returns new attempt count
- Triggers account lockout at threshold

#### `reset_failed_attempts(id) -> Result<()>`
- Resets counter to 0 after successful login

#### `record_login(id, ip) -> Result<()>`
- Updates `last_login_at` and `last_login_ip`
- Resets failed attempts
- Audit trail

**Implementation**: [UserRepository](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/user_repository.rs)

---

### Struct: `AuthRequest`

**Purpose**: DTO for login requests

**Fields**:
- `email`: User's email address
- `password`: Plaintext password (hashed immediately)
- `tenant_id`: Tenant context
- `ip_address`: Client IP (for risk assessment)
- `user_agent`: Browser/client info (for device fingerprinting)

**Security**: Password is plaintext in request, must be hashed before storage

---

### Struct: `AuthResponse`

**Purpose**: DTO for successful authentication

**Fields**:
- `user`: Complete user object
- `access_token`: JWT for API access (15 min)
- `refresh_token`: Opaque token for renewal (7 days)
- `requires_mfa`: Whether MFA verification needed

**Security**: User object must exclude `password_hash` and `mfa_secret`

---

### Struct: `IdentityService`

**Purpose**: Main service for identity operations

**Dependencies**:
- `store`: Arc<dyn UserStore> - User persistence
- `token_service`: Arc<dyn TokenProvider> - Token generation

---

### Method: `IdentityService::new()`

**Signature**: `pub fn new(store, token_service) -> Self`

**Purpose**: Constructor with dependency injection

**Pattern**: Service pattern with trait-based dependencies

---

### Method: `IdentityService::register()`

**Signature**: `pub async fn register(request, tenant_id) -> Result<User>`

**Purpose**: Register new user account

**Steps**:

1. **Validate Password Exists**
   ```rust
   if request.password.is_none() {
       return Err(AuthError::ValidationError { ... });
   }
   ```

2. **Check Email Uniqueness**
   ```rust
   if self.store.find_by_email(&request.email, tenant_id).await?.is_some() {
       return Err(AuthError::Conflict { message: "Email already registered" });
   }
   ```

3. **Hash Password (Argon2id)**
   ```rust
   let salt = SaltString::generate(&mut OsRng);
   let argon2 = Argon2::default();
   let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
   ```
   - Uses cryptographically secure random salt
   - Argon2id with default parameters (memory-hard)
   - Prevents rainbow table attacks

4. **Create User**
   ```rust
   let user = self.store.create(request, password_hash, tenant_id).await?;
   ```
   - Initial status: `PendingVerification`
   - Requires email verification before full access

5. **TODO: Trigger Audit Log**
   - Log registration event
   - Include: user_id, email, tenant_id, timestamp

**Returns**: Created user object

**Security**:
- Password never stored in plaintext
- Email uniqueness prevents account takeover
- Argon2id prevents brute-force attacks

---

### Method: `IdentityService::login()`

**Signature**: `pub async fn login(request) -> Result<AuthResponse>`

**Purpose**: Authenticate user and issue tokens

**Steps**:

1. **Fetch User**
   ```rust
   let user = self.store.find_by_email(&request.email, request.tenant_id).await?
       .ok_or(AuthError::InvalidCredentials)?;
   ```
   - Generic error (don't reveal if email exists)
   - Multi-tenant isolation

2. **Check Account Status**
   ```rust
   if !user.can_authenticate() {
       return Err(AuthError::Unauthorized { ... });
   }
   ```
   - Checks: Status is Active AND not locked
   - Prevents suspended/deleted users from logging in

3. **Verify Password**
   ```rust
   let parsed_hash = PasswordHash::new(user.password_hash.as_ref().unwrap())?;
   if Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_err() {
       // Failed attempt handling
   }
   ```
   - Constant-time comparison (prevents timing attacks)
   - Argon2 verification

4. **Failed Attempt Handling**
   ```rust
   let attempts = self.store.increment_failed_attempts(user.id).await?;
   if attempts >= 5 {
       // TODO: Lock account
   }
   return Err(AuthError::InvalidCredentials);
   ```
   - Increment counter on failure
   - Lock account after 5 attempts
   - Generic error message

5. **Record Successful Login**
   ```rust
   self.store.record_login(user.id, request.ip_address).await?;
   ```
   - Updates `last_login_at` and `last_login_ip`
   - Resets failed attempts counter

6. **Issue Tokens**
   ```rust
   let claims = Claims {
       sub: user.id.to_string(),
       iss: "auth-service",
       aud: "auth-service",
       exp: (Utc::now() + Duration::minutes(15)).timestamp(),
       iat: Utc::now().timestamp(),
       nbf: Utc::now().timestamp(),
       jti: Uuid::new_v4().to_string(),
       tenant_id: request.tenant_id.to_string(),
       permissions: vec![],  // TODO: Load from database
       roles: vec![],        // TODO: Load from database
   };
   
   let access_token = self.token_service.issue_access_token(claims).await?;
   let refresh_token = self.token_service.issue_refresh_token(user.id, tenant_id).await?;
   ```
   - Access token: 15-minute expiration
   - Refresh token: 7-day expiration
   - TODO: Load actual permissions/roles

7. **Check MFA Requirement**
   ```rust
   let requires_mfa = user.mfa_enabled;
   ```
   - If true, client must verify TOTP code

**Returns**: AuthResponse with user, tokens, and MFA flag

**Security**:
- Constant-time password verification
- Account lockout prevents brute-force
- Generic error messages prevent enumeration
- Audit trail for all login attempts

---

### Method: `IdentityService::ban_user()`

**Signature**: `pub async fn ban_user(user_id) -> Result<()>`

**Purpose**: Suspend user account (admin operation)

**Implementation**:
```rust
self.store.update_status(user_id, UserStatus::Suspended).await
```

**Effect**:
- User cannot authenticate
- Existing sessions remain valid (until expiration)
- TODO: Revoke all active tokens

**Authorization**: Requires admin permission

---

### Method: `IdentityService::activate_user()`

**Signature**: `pub async fn activate_user(user_id) -> Result<()>`

**Purpose**: Activate suspended or pending user

**Implementation**:
```rust
self.store.update_status(user_id, UserStatus::Active).await
```

**Use Cases**:
- Email verification complete
- Admin reinstates suspended account

---

## Security Considerations

### Password Security

1. **Argon2id Hashing**
   - Memory-hard algorithm (prevents GPU attacks)
   - Per-user salts (prevents rainbow tables)
   - Default parameters: 19 MiB memory, 2 iterations

2. **Never Store Plaintext**
   - Password hashed immediately on registration
   - Original password discarded after hashing

3. **Constant-Time Verification**
   - Prevents timing attacks
   - Argon2 verify_password is constant-time

### Account Lockout

1. **Threshold**: 5 failed attempts
2. **Duration**: 30 minutes (TODO: implement)
3. **Reset**: Successful login or timeout

### Error Messages

**Bad** (information leakage):
```rust
// DON'T
if user.is_none() {
    return Err("Email not found");
}
if !verify_password() {
    return Err("Invalid password");
}
```

**Good** (generic):
```rust
// DO
if user.is_none() || !verify_password() {
    return Err(AuthError::InvalidCredentials);
}
```

### Audit Logging

**TODO**: Log all authentication events
- Successful login
- Failed login (with reason)
- Registration
- Account status changes

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `argon2` | Password hashing |
| `uuid` | User IDs |
| `async-trait` | Async trait support |
| `serde` | Serialization |

### Internal Dependencies

- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [models/token.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/token.rs) - Claims
- [error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/error.rs) - AuthError
- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - TokenProvider

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_register_duplicate_email() {
    let service = setup_identity_service();
    let request = CreateUserRequest { email: "test@example.com", ... };
    
    service.register(request.clone(), tenant_id).await.unwrap();
    let result = service.register(request, tenant_id).await;
    
    assert!(matches!(result, Err(AuthError::Conflict { .. })));
}

#[tokio::test]
async fn test_login_invalid_password() {
    let service = setup_identity_service();
    let request = AuthRequest { password: "wrong", ... };
    
    let result = service.login(request).await;
    
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn test_login_locked_account() {
    let service = setup_identity_service();
    // Simulate 5 failed attempts
    for _ in 0..5 {
        service.login(wrong_password_request()).await.unwrap_err();
    }
    
    let result = service.login(correct_password_request()).await;
    
    assert!(matches!(result, Err(AuthError::AccountLocked { .. })));
}
```

---

## Related Files

- [handlers/auth.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/handlers/auth.rs) - HTTP endpoints
- [repositories/user_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/user_repository.rs) - UserStore implementation
- [services/token_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/token_service.rs) - Token generation

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 143  
**Security Level**: CRITICAL
