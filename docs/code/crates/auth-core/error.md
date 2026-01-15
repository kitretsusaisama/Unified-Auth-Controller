# error.rs

## File Metadata

**File Path**: `crates/auth-core/src/error.rs`  
**Crate**: `auth-core`  
**Module**: `error`  
**Layer**: Domain (Error Definitions)  
**Security-Critical**: ⚠️ **MEDIUM** - Error handling and information disclosure

## Purpose

Defines comprehensive error types for the authentication system using `thiserror` for ergonomic error handling and clear error messages.

### Problem It Solves

- Typed error handling
- Clear error messages
- Error conversion from external crates
- Structured error information
- Error propagation

---

## Detailed Code Breakdown

### Enum: `AuthError`

**Purpose**: Main error type for authentication operations

**Variants**:

#### 1. Authentication Errors

```rust
#[error("Authentication failed: {reason}")]
AuthenticationFailed { reason: String }
```

**Use Case**: Login failures, invalid credentials

---

```rust
#[error("Invalid credentials")]
InvalidCredentials
```

**Use Case**: Wrong username/password

---

#### 2. Authorization Errors

```rust
#[error("Authorization denied: {permission} on {resource}")]
AuthorizationDenied { permission: String, resource: String }
```

**Use Case**: RBAC/ABAC permission denied

---

```rust
#[error("Unauthorized: {message}")]
Unauthorized { message: String }
```

**Use Case**: Generic unauthorized access

---

#### 3. Token Errors

```rust
#[error("Token error: {kind:?}")]
TokenError { kind: TokenErrorKind }
```

**Use Case**: JWT validation failures

---

#### 4. Rate Limiting

```rust
#[error("Rate limit exceeded: {limit} requests per {window}")]
RateLimitExceeded { limit: u32, window: String }
```

**Use Case**: Too many requests

---

#### 5. Resource Errors

```rust
#[error("Tenant not found: {tenant_id}")]
TenantNotFound { tenant_id: String }
```

---

```rust
#[error("User not found: {user_id}")]
UserNotFound { user_id: String }
```

---

#### 6. Configuration Errors

```rust
#[error("Configuration error: {message}")]
ConfigurationError { message: String }
```

**Use Case**: Invalid configuration

---

#### 7. External Service Errors

```rust
#[error("External service error: {service} - {error}")]
ExternalServiceError { service: String, error: String }
```

**Use Case**: Third-party API failures

---

#### 8. Database Errors

```rust
#[error("Database error: {0}")]
DatabaseError(String)
```

**Use Case**: SQL errors, connection failures

---

#### 9. Validation Errors

```rust
#[error("Validation error: {message}")]
ValidationError { message: String }
```

---

```rust
#[error("Conflict: {message}")]
Conflict { message: String }
```

**Use Case**: Duplicate email, username conflicts

---

#### 10. Credential Errors

```rust
#[error("Credential error: {message}")]
CredentialError { message: String }
```

---

```rust
#[error("Password policy violation: {errors:?}")]
PasswordPolicyViolation { errors: Vec<String> }
```

**Use Case**: Password doesn't meet policy

---

```rust
#[error("Account locked: {reason}")]
AccountLocked { reason: String }
```

**Use Case**: Too many failed attempts

---

```rust
#[error("Password expired")]
PasswordExpired
```

---

#### 11. Internal Errors

```rust
#[error("Internal error: {0}")]
InternalError(String)
```

---

```rust
#[error("Crypto error: {0}")]
CryptoError(String)
```

---

### Enum: `TokenErrorKind`

**Purpose**: Specific token error types

**Variants**:

| Variant | Description |
|---------|-------------|
| `Expired` | Token past expiration |
| `Invalid` | Malformed token |
| `Revoked` | Token in blacklist |
| `MalformedSignature` | Invalid signature |
| `UnsupportedAlgorithm` | Algorithm not supported |

---

## Error Conversions

### From `sqlx::Error`

```rust
impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}
```

**Usage**:
```rust
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
    .fetch_one(&pool)
    .await?; // Automatically converts to AuthError
```

---

### From `validator::ValidationErrors`

```rust
impl From<validator::ValidationErrors> for AuthError {
    fn from(err: validator::ValidationErrors) -> Self {
        AuthError::ValidationError {
            message: err.to_string(),
        }
    }
}
```

**Usage**:
```rust
request.validate()?; // Automatically converts to AuthError
```

---

## Usage Examples

### Example 1: Authentication Failure

```rust
pub async fn login(email: &str, password: &str) -> Result<User, AuthError> {
    let user = find_user_by_email(email).await?;
    
    if !verify_password(password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }
    
    if user.status == UserStatus::Locked {
        return Err(AuthError::AccountLocked {
            reason: "Too many failed login attempts".to_string(),
        });
    }
    
    Ok(user)
}
```

---

### Example 2: Authorization Failure

```rust
pub async fn delete_document(
    user_id: Uuid,
    document_id: Uuid,
) -> Result<(), AuthError> {
    if !has_permission(user_id, "documents:delete").await? {
        return Err(AuthError::AuthorizationDenied {
            permission: "documents:delete".to_string(),
            resource: document_id.to_string(),
        });
    }
    
    delete(document_id).await
}
```

---

### Example 3: Token Validation

```rust
pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    match decode_jwt(token) {
        Ok(claims) => {
            if claims.exp < Utc::now().timestamp() {
                return Err(AuthError::TokenError {
                    kind: TokenErrorKind::Expired,
                });
            }
            Ok(claims)
        }
        Err(_) => Err(AuthError::TokenError {
            kind: TokenErrorKind::Invalid,
        }),
    }
}
```

---

### Example 4: Password Policy Violation

```rust
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    let mut errors = Vec::new();
    
    if password.len() < 12 {
        errors.push("Password must be at least 12 characters".to_string());
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain uppercase letter".to_string());
    }
    
    if !errors.is_empty() {
        return Err(AuthError::PasswordPolicyViolation { errors });
    }
    
    Ok(())
}
```

---

### Example 5: Rate Limiting

```rust
pub async fn check_rate_limit(user_id: Uuid) -> Result<(), AuthError> {
    let count = get_request_count(user_id).await?;
    
    if count > 100 {
        return Err(AuthError::RateLimitExceeded {
            limit: 100,
            window: "1 minute".to_string(),
        });
    }
    
    Ok(())
}
```

---

## Error Handling Patterns

### Pattern 1: Error Propagation

```rust
pub async fn register_user(req: RegisterRequest) -> Result<User, AuthError> {
    // Validation errors auto-convert
    req.validate()?;
    
    // Database errors auto-convert
    let user = create_user(req).await?;
    
    Ok(user)
}
```

---

### Pattern 2: Error Mapping

```rust
pub async fn external_api_call() -> Result<Data, AuthError> {
    reqwest::get("https://api.example.com/data")
        .await
        .map_err(|e| AuthError::ExternalServiceError {
            service: "example-api".to_string(),
            error: e.to_string(),
        })?
        .json()
        .await
        .map_err(|e| AuthError::ExternalServiceError {
            service: "example-api".to_string(),
            error: e.to_string(),
        })
}
```

---

### Pattern 3: Context Addition

```rust
pub async fn get_user(id: Uuid) -> Result<User, AuthError> {
    find_user_by_id(id)
        .await?
        .ok_or_else(|| AuthError::UserNotFound {
            user_id: id.to_string(),
        })
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `thiserror` | Error derive macros |
| `sqlx` | Database error conversion |
| `validator` | Validation error conversion |

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/lib.md) - Crate root
- [api/error.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/error.md) - HTTP error mapping

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 86  
**Security Level**: MEDIUM
