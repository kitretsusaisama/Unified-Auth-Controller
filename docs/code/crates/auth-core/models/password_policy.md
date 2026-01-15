# models/password_policy.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/password_policy.rs`  
**Crate**: `auth-core`  
**Module**: `models::password_policy`  
**Layer**: Domain (Data Model)  
**Security-Critical**: ✅ **YES** - Password security enforcement

## Purpose

Defines password policy configuration and rules for enforcing password strength, complexity, aging, and lockout policies across the platform.

### Problem It Solves

- Configurable password requirements
- Multi-tier security policies (basic, enterprise, high-security, compliance)
- Password aging and history enforcement
- Account lockout configuration
- Compliance with security standards (HIPAA, PCI-DSS)

---

## Detailed Code Breakdown

### Struct: `PasswordPolicyConfig`

**Purpose**: Complete password policy configuration

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Policy identifier |
| `tenant_id` | `Option<Uuid>` | Tenant-specific (None = global) |
| `name` | `String` | Policy name |
| `description` | `Option<String>` | Policy description |
| `policy` | `PasswordPolicyRules` | Actual rules |
| `is_active` | `bool` | Active status |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `updated_at` | `DateTime<Utc>` | Last update |

---

### Struct: `PasswordPolicyRules`

**Purpose**: Detailed password policy rules

**Categories**:

#### 1. Length Requirements
```rust
pub min_length: usize,      // Minimum password length
pub max_length: usize,      // Maximum password length
```

#### 2. Character Requirements
```rust
pub require_uppercase: bool,        // Must have uppercase
pub require_lowercase: bool,        // Must have lowercase
pub require_numbers: bool,          // Must have numbers
pub require_special_chars: bool,    // Must have special chars
pub min_special_chars: usize,       // Minimum special chars count
```

#### 3. Complexity Requirements
```rust
pub min_character_classes: usize,      // Different char types required
pub disallow_common_passwords: bool,   // Block common passwords
pub disallow_personal_info: bool,      // Block personal info
pub disallow_repeated_chars: bool,     // Block repeated chars (aaa)
pub disallow_sequential_chars: bool,   // Block sequential (abc, 123)
```

#### 4. Aging and History
```rust
pub max_age_days: Option<u32>,        // Password expiration
pub history_count: usize,             // Remember N passwords
pub min_age_hours: Option<u32>,       // Min time before change
```

#### 5. Lockout Policy
```rust
pub lockout_threshold: u32,                    // Failed attempts before lock
pub lockout_duration_minutes: u32,             // Lock duration
pub lockout_reset_time_minutes: Option<u32>,   // Reset failed attempts
```

#### 6. Advanced Features
```rust
pub require_mfa_for_privileged: bool,  // MFA for admins
pub password_strength_meter: bool,     // Show strength meter
pub custom_dictionary: Vec<String>,    // Additional forbidden words
```

---

## Default Policy (Enterprise Grade)

```rust
impl Default for PasswordPolicyRules {
    fn default() -> Self {
        Self {
            // Length: 12-128 characters
            min_length: 12,
            max_length: 128,
            
            // All character types required
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            min_special_chars: 2,
            
            // Strong complexity
            min_character_classes: 3,
            disallow_common_passwords: true,
            disallow_personal_info: true,
            disallow_repeated_chars: true,
            disallow_sequential_chars: true,
            
            // 90-day expiration, 12 password history
            max_age_days: Some(90),
            history_count: 12,
            min_age_hours: Some(24),
            
            // 5 attempts, 30-minute lockout
            lockout_threshold: 5,
            lockout_duration_minutes: 30,
            lockout_reset_time_minutes: Some(60),
            
            // Advanced security
            require_mfa_for_privileged: true,
            password_strength_meter: true,
            custom_dictionary: Vec::new(),
        }
    }
}
```

---

## Policy Templates

### 1. Basic Policy (Low Security)

```rust
PasswordPolicyTemplates::basic()
```

**Configuration**:
- Min length: 8 characters
- Requires: uppercase, lowercase, numbers
- No special characters required
- 180-day expiration
- 3 password history
- 10 failed attempts before lockout

**Use Case**: Internal tools, development environments

---

### 2. Enterprise Policy (Default)

```rust
PasswordPolicyTemplates::enterprise()
```

**Configuration**: Same as `Default`

**Use Case**: Standard business applications

---

### 3. High Security Policy

```rust
PasswordPolicyTemplates::high_security()
```

**Configuration**:
- Min length: 16 characters
- Requires: all character types
- Min 3 special characters
- 60-day expiration
- 24 password history
- 48-hour minimum age
- 3 failed attempts before lockout

**Use Case**: Financial systems, privileged accounts

---

### 4. Compliance Policy (HIPAA, PCI-DSS)

```rust
PasswordPolicyTemplates::compliance()
```

**Configuration**:
- Min length: 14 characters
- All character types required
- 90-day expiration
- 12 password history
- 24-hour minimum age
- 5 failed attempts before lockout

**Use Case**: Healthcare, payment processing, regulated industries

---

## Methods

### Method: `PasswordPolicyConfig::new()`

**Signature**: `pub fn new(tenant_id: Option<Uuid>, name: String, policy: PasswordPolicyRules) -> Self`

**Purpose**: Create new policy configuration

**Example**:
```rust
let policy = PasswordPolicyConfig::new(
    Some(tenant_id),
    "Custom Enterprise Policy".to_string(),
    PasswordPolicyTemplates::enterprise(),
);
```

---

### Method: `is_more_restrictive_than()`

**Signature**: `pub fn is_more_restrictive_than(&self, other: &PasswordPolicyRules) -> bool`

**Purpose**: Compare policy strictness

**Logic**:
```rust
self.policy.min_length >= other.min_length
    && self.policy.min_special_chars >= other.min_special_chars
    && self.policy.lockout_threshold <= other.lockout_threshold
    && self.policy.history_count >= other.history_count
```

**Use Case**: Policy inheritance, ensuring child policies don't weaken parent

---

### Method: `get_effective_policy()`

**Signature**: `pub fn get_effective_policy(&self) -> PasswordPolicyRules`

**Purpose**: Get effective policy with inheritance

**Future Enhancement**: Merge tenant → organization → global policies

---

## Usage Examples

### Example 1: Create Custom Policy

```rust
let custom_policy = PasswordPolicyRules {
    min_length: 10,
    max_length: 64,
    require_uppercase: true,
    require_lowercase: true,
    require_numbers: true,
    require_special_chars: false,
    min_special_chars: 0,
    min_character_classes: 3,
    disallow_common_passwords: true,
    disallow_personal_info: false,
    disallow_repeated_chars: true,
    disallow_sequential_chars: true,
    max_age_days: Some(120),
    history_count: 6,
    min_age_hours: None,
    lockout_threshold: 8,
    lockout_duration_minutes: 20,
    lockout_reset_time_minutes: Some(45),
    require_mfa_for_privileged: false,
    password_strength_meter: true,
    custom_dictionary: vec!["company".to_string(), "product".to_string()],
};

let config = PasswordPolicyConfig::new(
    Some(tenant_id),
    "Custom Policy".to_string(),
    custom_policy,
);
```

---

### Example 2: Policy Hierarchy

```rust
// Global policy (strictest)
let global_policy = PasswordPolicyTemplates::high_security();

// Organization policy (inherits from global)
let org_policy = PasswordPolicyTemplates::enterprise();

// Tenant policy (inherits from org)
let tenant_policy = PasswordPolicyTemplates::basic();

// Validate hierarchy
let global_config = PasswordPolicyConfig::new(None, "Global".to_string(), global_policy);
let org_config = PasswordPolicyConfig::new(Some(org_id), "Org".to_string(), org_policy);

if !org_config.is_more_restrictive_than(&global_config.policy) {
    return Err(AuthError::ValidationError {
        message: "Organization policy must be at least as restrictive as global policy".to_string(),
    });
}
```

---

### Example 3: Tenant-Specific Policies

```rust
// Healthcare tenant - compliance policy
let healthcare_policy = PasswordPolicyConfig::new(
    Some(healthcare_tenant_id),
    "HIPAA Compliance".to_string(),
    PasswordPolicyTemplates::compliance(),
);

// Startup tenant - basic policy
let startup_policy = PasswordPolicyConfig::new(
    Some(startup_tenant_id),
    "Startup Basic".to_string(),
    PasswordPolicyTemplates::basic(),
);

// Financial tenant - high security
let financial_policy = PasswordPolicyConfig::new(
    Some(financial_tenant_id),
    "Financial High Security".to_string(),
    PasswordPolicyTemplates::high_security(),
);
```

---

## Integration with Credential Service

```rust
use crate::services::credential::CredentialService;

// Create credential service with policy
let credential_service = CredentialService::new(Some(policy.policy));

// Validate password
let validation = credential_service.validate_password("MyP@ssw0rd123");
if !validation.is_valid {
    return Err(AuthError::PasswordPolicyViolation {
        errors: validation.errors,
    });
}
```

---

## Database Schema

```sql
CREATE TABLE password_policies (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    policy JSON NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_is_active (is_active),
    UNIQUE KEY unique_tenant_name (tenant_id, name),
    
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `serde` | Serialization |
| `uuid` | Identifiers |
| `chrono` | Timestamps |
| `validator` | Validation |

---

## Related Files

- [services/credential.md](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/credential.rs) - Password validation
- [models/user.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/user.md) - User model

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 235  
**Security Level**: CRITICAL
