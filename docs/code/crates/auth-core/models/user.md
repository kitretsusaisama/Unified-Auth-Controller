# models/user.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/user.rs`  
**Crate**: `auth-core`  
**Module**: `models::user`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Contains sensitive user data including password hashes, MFA secrets, and PII

## Purpose

This file defines the core `User` entity and related types for the authentication system. It represents the central domain model for user identity, authentication state, and security attributes.

### Problem It Solves

- Provides a type-safe representation of user data
- Enforces validation rules for user attributes
- Implements business logic for authentication state (locked, active, verified)
- Manages security-critical fields (password hash, MFA secrets, backup codes)

### How It Fits

The `User` model is the foundation of the identity management system. It's used by:
- `IdentityService` for authentication and registration
- `UserRepository` for database persistence
- `AuthorizationService` for permission checks
- `RiskEngine` for risk assessment

---

## Detailed Code Breakdown

### Struct: `User`

**Purpose**: Core user entity representing an authenticated identity

**Fields**:

| Field | Type | Description | Security Notes |
|-------|------|-------------|----------------|
| `id` | `Uuid` | Unique user identifier | Primary key, immutable |
| `email` | `String` | User's email address | **PII**, validated with `#[validate(email)]` |
| `email_verified` | `bool` | Email verification status | Security: Prevents unauthorized access |
| `phone` | `Option<String>` | Phone number | **PII**, optional |
| `phone_verified` | `bool` | Phone verification status | For SMS-based MFA |
| `password_hash` | `Option<String>` | Argon2id password hash | **CRITICAL**: Never expose, optional for OAuth users |
| `password_changed_at` | `Option<DateTime<Utc>>` | Last password change timestamp | For password expiration policies |
| `failed_login_attempts` | `u32` | Failed login counter | Security: Account lockout mechanism |
| `locked_until` | `Option<DateTime<Utc>>` | Account lockout expiration | Security: Prevents brute-force attacks |
| `last_login_at` | `Option<DateTime<Utc>>` | Last successful login time | Audit trail |
| `last_login_ip` | `Option<String>` | Last login IP address | Security: Anomaly detection |
| `mfa_enabled` | `bool` | MFA enrollment status | Security: Two-factor authentication |
| `mfa_secret` | `Option<String>` | TOTP secret key | **CRITICAL**: Never expose, encrypted at rest |
| `backup_codes` | `Option<Vec<String>>` | MFA backup codes | **CRITICAL**: Hashed before storage |
| `risk_score` | `f32` | User risk score (0.0-1.0) | Security: Adaptive authentication |
| `profile_data` | `serde_json::Value` | Custom profile attributes | Flexible schema, **may contain PII** |
| `preferences` | `serde_json::Value` | User preferences | UI settings, notifications |
| `status` | `UserStatus` | Account status | Active, Suspended, Deleted, PendingVerification |
| `created_at` | `DateTime<Utc>` | Account creation timestamp | Audit trail |
| `updated_at` | `DateTime<Utc>` | Last update timestamp | Audit trail |
| `deleted_at` | `Option<DateTime<Utc>>` | Soft delete timestamp | GDPR compliance: Right to erasure |

**Derive Macros**:
- `Debug`: Debugging support (⚠️ **WARNING**: May leak secrets in logs)
- `Clone`: Cheap cloning for Arc-wrapped instances
- `Serialize`, `Deserialize`: JSON serialization for API responses
- `Validate`: Automatic validation via `validator` crate
- `utoipa::ToSchema`: OpenAPI schema generation

**Validation Rules**:
- `email`: RFC 5322 email validation

**Security Considerations**:
- **Password Hash**: Must never be included in API responses
- **MFA Secret**: Must never be exposed, encrypted in database
- **Backup Codes**: Must be hashed before storage
- **PII Fields**: Email, phone, profile_data require GDPR compliance

---

### Enum: `UserStatus`

**Purpose**: Represents the lifecycle state of a user account

**Variants**:

1. **`Active`**
   - User can authenticate and access resources
   - Normal operational state

2. **`Suspended`**
   - Account temporarily disabled
   - Reasons: Policy violation, security concern, admin action
   - User cannot authenticate

3. **`Deleted`**
   - Soft delete state
   - `deleted_at` timestamp set
   - User cannot authenticate
   - Data retained for audit/compliance

4. **`PendingVerification`**
   - New account awaiting email verification
   - User cannot fully authenticate until verified
   - **Default state** for new registrations

**Database Mapping**:
- `#[sqlx(rename_all = "snake_case")]`: Maps to lowercase snake_case in database
- Example: `Active` → `"active"` in MySQL

**Security**:
- Status transitions must be logged to audit trail
- Only admins can change status (except self-initiated deletion)

---

### Struct: `CreateUserRequest`

**Purpose**: DTO for user registration requests

**Fields**:
- `email`: String (validated as email)
- `phone`: Option<String> (optional phone number)
- `password`: Option<String> (8-128 characters, optional for OAuth users)
- `profile_data`: Option<serde_json::Value> (custom profile attributes)

**Validation Rules**:
- `email`: Must be valid RFC 5322 email
- `password`: Length between 8-128 characters (if provided)

**Usage**:
```rust
let request = CreateUserRequest {
    email: "user@example.com".to_string(),
    phone: Some("+1234567890".to_string()),
    password: Some("SecurePassword123".to_string()),
    profile_data: Some(json!({"first_name": "John", "last_name": "Doe"})),
};

// Validate before processing
request.validate()?;
```

**Security**:
- Password is plaintext in request (must be hashed immediately)
- Validation prevents empty/invalid emails
- Profile data should be sanitized to prevent XSS

---

### Struct: `UpdateUserRequest`

**Purpose**: DTO for user profile update requests

**Fields**:
- `id`: Uuid (user to update)
- `email`: Option<String> (new email, requires re-verification)
- `phone`: Option<String> (new phone number)
- `profile_data`: Option<serde_json::Value> (updated profile)
- `preferences`: Option<serde_json::Value> (updated preferences)

**Validation Rules**:
- `email`: Must be valid RFC 5322 email (if provided)

**Security**:
- Email changes must trigger re-verification
- Authorization check required (user can only update own profile, or admin)
- Audit log required for all updates

---

### Method: `User::is_locked()`

**Signature**: `pub fn is_locked(&self) -> bool`

**Purpose**: Check if user account is currently locked due to failed login attempts

**Logic**:
```rust
if let Some(locked_until) = self.locked_until {
    locked_until > Utc::now()  // Locked if lockout hasn't expired
} else {
    false  // Not locked if no lockout set
}
```

**Returns**:
- `true`: Account is locked (lockout period hasn't expired)
- `false`: Account is not locked

**Side Effects**: None (pure function)

**Usage**:
```rust
if user.is_locked() {
    return Err(AuthError::AccountLocked {
        reason: "Too many failed login attempts".to_string()
    });
}
```

**Security**:
- Prevents brute-force attacks
- Lockout duration typically 30 minutes
- Automatic unlock after expiration

---

### Method: `User::can_authenticate()`

**Signature**: `pub fn can_authenticate(&self) -> bool`

**Purpose**: Check if user is allowed to authenticate

**Logic**:
```rust
matches!(self.status, UserStatus::Active) && !self.is_locked()
```

**Returns**:
- `true`: User can authenticate (Active status AND not locked)
- `false`: User cannot authenticate

**Conditions for `true`**:
1. Status is `Active` (not Suspended, Deleted, or PendingVerification)
2. Account is not locked

**Usage**:
```rust
if !user.can_authenticate() {
    return Err(AuthError::Unauthorized {
        message: "Account cannot authenticate".to_string()
    });
}
```

**Security**:
- Central authorization check for all authentication flows
- Must be called before password verification

---

### Method: `User::is_email_verified()`

**Signature**: `pub fn is_email_verified(&self) -> bool`

**Purpose**: Check if user's email address has been verified

**Returns**: `self.email_verified`

**Usage**:
```rust
if !user.is_email_verified() {
    // Send verification email
    // Restrict access to certain features
}
```

**Security**:
- Email verification prevents unauthorized account creation
- Some features may require verified email

---

### Method: `User::get_risk_score()`

**Signature**: `pub fn get_risk_score(&self) -> f32`

**Purpose**: Get user's risk score, clamped to valid range

**Logic**:
```rust
self.risk_score.clamp(0.0, 1.0)
```

**Returns**: Risk score between 0.0 (low risk) and 1.0 (high risk)

**Usage**:
```rust
let risk = user.get_risk_score();
if risk > 0.7 {
    // Trigger MFA
    requires_mfa = true;
}
```

**Risk Score Interpretation**:
- `0.0 - 0.3`: Low risk (normal authentication)
- `0.3 - 0.5`: Medium risk (log warning)
- `0.5 - 0.7`: Elevated risk (email verification)
- `0.7 - 0.9`: High risk (require MFA)
- `0.9 - 1.0`: Critical risk (block login, manual review)

---

### Impl: `Default for UserStatus`

**Purpose**: Provide default status for new users

**Returns**: `UserStatus::PendingVerification`

**Rationale**:
- New users must verify email before full access
- Security best practice: Verify identity before granting access
- Prevents spam/bot registrations

---

## Dependencies

### External Crates

| Crate | Purpose | Why Needed |
|-------|---------|------------|
| `chrono` | Date/time handling | Timestamps for created_at, updated_at, locked_until |
| `serde` | Serialization | JSON API responses, database serialization |
| `uuid` | Unique identifiers | User IDs, immutable primary keys |
| `validator` | Input validation | Email validation, password length checks |
| `utoipa` | OpenAPI schema | API documentation generation |
| `sqlx` | Database types | UserStatus enum mapping to database |

### Internal Dependencies

None (pure domain model)

---

## Testing

### Unit Tests

**Location**: Same file, `#[cfg(test)] mod tests`

**Test Cases**:
- `is_locked()` returns true when locked_until is in future
- `is_locked()` returns false when locked_until is in past
- `can_authenticate()` returns false for Suspended users
- `can_authenticate()` returns false for locked users
- `get_risk_score()` clamps values outside 0.0-1.0 range
- Email validation rejects invalid emails
- Password length validation enforces 8-128 characters

### Property-Based Tests

**Location**: `tests/property_tests.rs`

```rust
proptest! {
    #[test]
    fn test_risk_score_always_valid(score in any::<f32>()) {
        let user = User { risk_score: score, ..Default::default() };
        let clamped = user.get_risk_score();
        assert!(clamped >= 0.0 && clamped <= 1.0);
    }
}
```

---

## Security Checklist

- [x] Password hash never exposed in API responses
- [x] MFA secret encrypted at rest
- [x] Backup codes hashed before storage
- [x] Email validation prevents injection attacks
- [x] Account lockout prevents brute-force
- [x] Soft delete preserves audit trail
- [x] PII fields identified for GDPR compliance
- [x] Risk score used for adaptive authentication

---

## Related Files

- [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - Uses User for authentication
- [repositories/user_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/user_repository.rs) - Persists User to database
- [handlers/auth.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/handlers/auth.rs) - Exposes User via API
- [models/session.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/session.rs) - Links sessions to users

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 93  
**Security Level**: CRITICAL
