# validation.rs

## File Metadata

**File Path**: `crates/auth-api/src/validation.rs`  
**Crate**: `auth-api`  
**Module**: `validation`  
**Layer**: Adapter (Input Validation)  
**Security-Critical**: ✅ **YES** - Input validation and sanitization

## Purpose

Provides comprehensive input validation and sanitization functions for HTTP requests, preventing security vulnerabilities and ensuring data quality.

### Problem It Solves

- Password strength validation
- Email format validation and normalization
- Input sanitization (XSS prevention)
- Common password detection
- DoS prevention (length limits)

---

## Detailed Code Breakdown

### Static: `COMMON_PASSWORDS`

**Purpose**: List of commonly used weak passwords

**Passwords**: password, 123456, qwerty, abc123, etc. (24 total)

---

### Static: `EMAIL_REGEX`

**Purpose**: Email validation regex

**Pattern**: `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`

---

### Function: `validate_password()`

**Signature**: `pub fn validate_password(password: &str) -> Result<(), AuthError>`

**Purpose**: Validate password strength

**Checks**:

#### 1. Length Requirements
```rust
if password.len() < 12 {
    errors.push("Password must be at least 12 characters long");
}

if password.len() > 128 {
    errors.push("Password must not exceed 128 characters");
}
```

#### 2. Character Requirements
- Uppercase letter
- Lowercase letter
- Number
- Special character

#### 3. Common Password Detection
```rust
if COMMON_PASSWORDS.iter().any(|&common| lowercase_password.contains(common)) {
    errors.push("Password is too common or easily guessable");
}
```

**Example**:
```rust
// Valid password
validate_password("MyS3cur3P@ssw0rd!")?; // ✅

// Invalid passwords
validate_password("password123")?; // ❌ Common password
validate_password("Short1!")?; // ❌ Too short
validate_password("nouppercase123!")?; // ❌ No uppercase
```

---

### Function: `validate_email()`

**Signature**: `pub fn validate_email(email: &str) -> Result<String, AuthError>`

**Purpose**: Validate and normalize email

**Process**:

#### 1. Trim Whitespace
```rust
let trimmed = email.trim();
```

#### 2. Check Empty
```rust
if trimmed.is_empty() {
    return Err(AuthError::ValidationError {
        message: "Email cannot be empty".to_string(),
    });
}
```

#### 3. Check Length (RFC 5321)
```rust
if trimmed.len() > 254 {
    return Err(AuthError::ValidationError {
        message: "Email is too long".to_string(),
    });
}
```

#### 4. Validate Format
```rust
if !EMAIL_REGEX.is_match(trimmed) {
    return Err(AuthError::ValidationError {
        message: "Invalid email format".to_string(),
    });
}
```

#### 5. Normalize to Lowercase
```rust
Ok(trimmed.to_lowercase())
```

**Example**:
```rust
let email = validate_email("  USER@EXAMPLE.COM  ")?;
assert_eq!(email, "user@example.com");
```

---

### Function: `sanitize_input()`

**Signature**: `pub fn sanitize_input(input: &str) -> String`

**Purpose**: Remove control characters to prevent XSS

**Process**:
```rust
input
    .chars()
    .filter(|c| !c.is_control() || c.is_whitespace())
    .collect()
```

**Example**:
```rust
let clean = sanitize_input("Hello\x00World\x1B");
assert_eq!(clean, "HelloWorld");
```

---

## Usage Examples

### Example 1: Registration Validation

```rust
pub async fn register(
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>> {
    // Validate email
    let email = validate_email(&payload.email)?;
    
    // Validate password
    validate_password(&payload.password)?;
    
    // Sanitize name
    let full_name = payload.full_name.map(|n| sanitize_input(&n));
    
    // Create user
    let user = user_repo.create(CreateUserRequest {
        email,
        full_name,
        ..Default::default()
    }).await?;
    
    Ok(Json(user))
}
```

---

### Example 2: Password Change Validation

```rust
pub async fn change_password(
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode> {
    // Validate new password
    validate_password(&payload.new_password)?;
    
    // Ensure new password is different
    if payload.new_password == payload.current_password {
        return Err(AuthError::ValidationError {
            message: "New password must be different".to_string(),
        });
    }
    
    // Update password
    update_password(user_id, &payload.new_password).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

---

## Security Considerations

### 1. DoS Prevention

**Length Limits**: Prevent excessive password lengths

```rust
if password.len() > 128 {
    return Err(AuthError::ValidationError {
        message: "Password too long".to_string(),
    });
}
```

### 2. Common Password Detection

**Prevents**: Use of easily guessable passwords

### 3. XSS Prevention

**Sanitization**: Remove control characters from user input

---

## Testing

### Unit Tests

```rust
#[test]
fn test_weak_password() {
    assert!(validate_password("password123").is_err());
    assert!(validate_password("12345678").is_err());
    assert!(validate_password("Short1!").is_err());
}

#[test]
fn test_strong_password() {
    assert!(validate_password("MyS3cur3P@ssw0rd!").is_ok());
    assert!(validate_password("C0mpl3x&Str0ng#Pass").is_ok());
}

#[test]
fn test_email_validation() {
    assert!(validate_email("user@example.com").is_ok());
    assert!(validate_email("  USER@EXAMPLE.COM  ").is_ok());
    assert!(validate_email("invalid-email").is_err());
    assert!(validate_email("@example.com").is_err());
}

#[test]
fn test_email_normalization() {
    let email = validate_email("  USER@EXAMPLE.COM  ").unwrap();
    assert_eq!(email, "user@example.com");
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `regex` | Email validation |
| `once_cell` | Lazy static initialization |

### Internal Dependencies

- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Auth handlers
- [services/credential.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/credential.md) - Password validation

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 140  
**Security Level**: CRITICAL
