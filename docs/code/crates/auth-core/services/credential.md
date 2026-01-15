# services/credential.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/credential.rs`  
**Crate**: `auth-core`  
**Module**: `services::credential`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - Password security and validation

## Purpose

Comprehensive credential management service providing password hashing, validation, policy enforcement, strength calculation, and account lockout management using Argon2id.

### Problem It Solves

- Secure password hashing (Argon2id)
- Password policy enforcement
- Password strength calculation
- Password aging and history
- Account lockout logic
- Common password detection

---

## Detailed Code Breakdown

### Struct: `CredentialService`

**Purpose**: Core credential management

**Fields**:
- `password_hasher`: `PasswordHasher` - Argon2id hasher
- `policy`: `PasswordPolicyRules` - Active policy

---

### Method: `CredentialService::new()`

**Signature**: `pub fn new(policy: Option<PasswordPolicyRules>) -> Self`

**Purpose**: Create service with custom or default policy

---

### Method: `with_template()`

**Signature**: `pub fn with_template(template: &str) -> Self`

**Purpose**: Create service with predefined policy template

**Templates**:
- `"basic"` - 8 chars, basic requirements
- `"enterprise"` - 12 chars, all requirements (default)
- `"high_security"` - 16 chars, strict requirements
- `"compliance"` - 14 chars, HIPAA/PCI-DSS compliant

**Example**:
```rust
let service = CredentialService::with_template("compliance");
```

---

## Password Operations

### Method: `hash_password()`

**Signature**: `pub fn hash_password(&self, password: &str) -> Result<String, AuthError>`

**Purpose**: Hash password using Argon2id

**Algorithm**: Argon2id (memory-hard, resistant to GPU attacks)

**Example**:
```rust
let hash = credential_service.hash_password("MyP@ssw0rd123")?;
// Returns: $argon2id$v=19$m=19456,t=2,p=1$...
```

---

### Method: `verify_password()`

**Signature**: `pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError>`

**Purpose**: Verify password against hash

**Example**:
```rust
if credential_service.verify_password(input_password, &user.password_hash)? {
    // Password correct
} else {
    // Password incorrect
}
```

---

## Password Validation

### Method: `validate_password()`

**Signature**: `pub fn validate_password(&self, password: &str) -> CredentialValidationResult`

**Purpose**: Comprehensive password validation

**Checks**:

#### 1. Length Requirements
```rust
if password.len() < self.policy.min_length {
    errors.push(format!("Password must be at least {} characters", min_length));
}
```

#### 2. Character Requirements
- Uppercase letters
- Lowercase letters
- Numbers
- Special characters (min count)

#### 3. Character Class Diversity
```rust
let character_classes = count_character_classes(password);
if character_classes < self.policy.min_character_classes {
    errors.push("Password must contain different types of characters");
}
```

#### 4. Pattern Detection
- Common passwords (password123, qwerty, etc.)
- Repeated characters (aaa, 111)
- Sequential characters (abc, 123)
- Custom dictionary words

**Returns**:
```rust
CredentialValidationResult {
    is_valid: bool,
    errors: Vec<String>,
    strength_score: u8, // 0-100
}
```

---

### Method: `calculate_password_strength()`

**Signature**: `pub fn calculate_password_strength(&self, password: &str) -> PasswordStrengthResult`

**Purpose**: Calculate password strength with feedback

**Scoring**:
- Length (20 points for min, +10 for 16+, +5 for 20+)
- Uppercase (15 points)
- Lowercase (15 points)
- Numbers (15 points)
- Special chars (15 points)
- Penalties for common patterns (-20), repeated chars (-10), sequential (-10)

**Returns**:
```rust
PasswordStrengthResult {
    score: u8, // 0-100
    feedback: Vec<String>,
    estimated_crack_time: String,
}
```

**Crack Time Estimates**:
- 0-20: "Seconds to minutes"
- 21-40: "Minutes to hours"
- 41-60: "Hours to days"
- 61-80: "Days to months"
- 81-100: "Months to years"

---

## Password Aging

### Method: `is_password_change_required()`

**Signature**: `pub fn is_password_change_required(&self, password_changed_at: Option<DateTime<Utc>>) -> bool`

**Purpose**: Check if password has expired

**Logic**:
```rust
if let (Some(max_age_days), Some(changed_at)) = (self.policy.max_age_days, password_changed_at) {
    Utc::now() - changed_at > Duration::days(max_age_days as i64)
} else {
    false
}
```

---

### Method: `can_change_password()`

**Signature**: `pub fn can_change_password(&self, password_changed_at: Option<DateTime<Utc>>) -> bool`

**Purpose**: Check minimum age before password change

**Use Case**: Prevent rapid password changes to bypass history

---

## Account Lockout

### Method: `should_lock_account()`

**Signature**: `pub fn should_lock_account(&self, failed_attempts: u32) -> bool`

**Purpose**: Determine if account should be locked

**Logic**:
```rust
failed_attempts >= self.policy.lockout_threshold
```

---

### Method: `calculate_unlock_time()`

**Signature**: `pub fn calculate_unlock_time(&self) -> DateTime<Utc>`

**Purpose**: Calculate when account should unlock

**Returns**: Current time + lockout duration

---

## Password History

### Method: `is_password_in_history()`

**Signature**: `pub fn is_password_in_history(&self, password: &str, history: &[PasswordHistoryEntry]) -> Result<bool, AuthError>`

**Purpose**: Check if password was previously used

**Logic**:
```rust
for entry in history.iter().take(self.policy.history_count) {
    if self.verify_password(password, &entry.password_hash)? {
        return Ok(true); // Password reused
    }
}
Ok(false)
```

---

## Pattern Detection

### Method: `has_common_patterns()`

**Purpose**: Detect common password patterns

**Common Patterns**:
```rust
["password", "123456", "qwerty", "admin", "letmein",
 "welcome", "monkey", "dragon", "master", "shadow",
 "login", "pass", "root", "user", "test", "guest"]
```

---

### Method: `has_repeated_chars()`

**Purpose**: Detect repeated characters

**Example**: "aaa", "111", "!!!"

---

### Method: `has_sequential_chars()`

**Purpose**: Detect sequential characters

**Examples**: "abc", "123", "xyz"

---

### Method: `contains_custom_words()`

**Purpose**: Check custom dictionary

**Use Case**: Block company name, product names, etc.

---

## Usage Examples

### Example 1: User Registration

```rust
pub async fn register_user(
    email: String,
    password: String,
    credential_service: &CredentialService,
) -> Result<User> {
    // Validate password
    let validation = credential_service.validate_password(&password);
    if !validation.is_valid {
        return Err(AuthError::PasswordPolicyViolation {
            errors: validation.errors,
        });
    }
    
    // Hash password
    let password_hash = credential_service.hash_password(&password)?;
    
    // Create user
    let user = User {
        id: Uuid::new_v4(),
        email,
        password_hash,
        password_changed_at: Some(Utc::now()),
        ..Default::default()
    };
    
    Ok(user)
}
```

---

### Example 2: Password Change

```rust
pub async fn change_password(
    user_id: Uuid,
    current_password: String,
    new_password: String,
    credential_service: &CredentialService,
    user_repo: &UserRepository,
) -> Result<()> {
    let user = user_repo.find_by_id(user_id).await?;
    
    // Verify current password
    if !credential_service.verify_password(&current_password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }
    
    // Check minimum age
    if !credential_service.can_change_password(user.password_changed_at) {
        return Err(AuthError::ValidationError {
            message: "Password changed too recently".to_string(),
        });
    }
    
    // Validate new password
    let validation = credential_service.validate_password(&new_password);
    if !validation.is_valid {
        return Err(AuthError::PasswordPolicyViolation {
            errors: validation.errors,
        });
    }
    
    // Check history
    let history = user_repo.get_password_history(user_id).await?;
    if credential_service.is_password_in_history(&new_password, &history)? {
        return Err(AuthError::PasswordPolicyViolation {
            errors: vec!["Password was previously used".to_string()],
        });
    }
    
    // Hash and update
    let new_hash = credential_service.hash_password(&new_password)?;
    user_repo.update_password(user_id, new_hash).await?;
    
    Ok(())
}
```

---

### Example 3: Login with Lockout

```rust
pub async fn login(
    email: String,
    password: String,
    credential_service: &CredentialService,
    user_repo: &UserRepository,
) -> Result<User> {
    let user = user_repo.find_by_email(&email).await?;
    
    // Check if locked
    if user.locked_until.map_or(false, |until| until > Utc::now()) {
        return Err(AuthError::AccountLocked {
            reason: "Too many failed attempts".to_string(),
        });
    }
    
    // Verify password
    if !credential_service.verify_password(&password, &user.password_hash)? {
        // Increment failed attempts
        let failed_attempts = user.failed_login_attempts + 1;
        user_repo.update_failed_attempts(user.id, failed_attempts).await?;
        
        // Check if should lock
        if credential_service.should_lock_account(failed_attempts) {
            let unlock_time = credential_service.calculate_unlock_time();
            user_repo.lock_account(user.id, unlock_time).await?;
            
            return Err(AuthError::AccountLocked {
                reason: format!("Account locked until {}", unlock_time),
            });
        }
        
        return Err(AuthError::InvalidCredentials);
    }
    
    // Check password expiration
    if credential_service.is_password_change_required(user.password_changed_at) {
        return Err(AuthError::PasswordExpired);
    }
    
    // Reset failed attempts
    user_repo.update_failed_attempts(user.id, 0).await?;
    
    Ok(user)
}
```

---

## Testing

### Unit Tests

```rust
#[test]
fn test_password_validation() {
    let service = CredentialService::new(None);
    
    // Weak password
    let result = service.validate_password("weak");
    assert!(!result.is_valid);
    
    // Strong password
    let result = service.validate_password("StrongP@ssw0rd246!Extra");
    assert!(result.is_valid);
    assert!(result.strength_score > 80);
}

#[test]
fn test_common_patterns() {
    let service = CredentialService::new(None);
    assert!(service.has_common_patterns("password123"));
    assert!(!service.has_common_patterns("MySecureP@ssw0rd"));
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `argon2` | Password hashing (production) |
| `validator` | Request validation |
| `chrono` | Timestamps |

### Internal Dependencies

- [models/password_policy.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/password_policy.md) - Password policy

---

## Related Files

- [models/password_policy.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/password_policy.md) - Password policy
- [models/user.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/user.md) - User model

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 430  
**Security Level**: CRITICAL
