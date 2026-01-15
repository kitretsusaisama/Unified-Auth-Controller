# validation.rs

## File Metadata

**File Path**: `crates/auth-config/src/validation.rs`  
**Crate**: `auth-config`  
**Module**: `validation`  
**Layer**: Infrastructure  
**Security-Critical**: ✅ **YES** - Validates security-critical configuration

## Purpose

Provides comprehensive validation for application configuration beyond basic type checking, ensuring security best practices and operational constraints are enforced.

### Problem It Solves

- Validates security settings (JWT secret strength, token expiry)
- Ensures database configuration is reasonable
- Prevents misconfiguration that could lead to security vulnerabilities
- Provides clear error messages for configuration issues

---

## Detailed Code Breakdown

### Enum: `ConfigValidationError`

**Purpose**: Specific error types for configuration validation failures

**Variants**:

1. **`ValidationFailed(ValidationErrors)`**
   - From `validator` crate
   - Basic field validation failures

2. **`SecurityValidationFailed { message: String }`**
   - Security-specific validation failures
   - JWT secret too weak, token expiry too long, etc.

3. **`DatabaseValidationFailed { message: String }`**
   - Database configuration issues
   - Connection pool misconfiguration

4. **`FeatureValidationFailed { message: String }`**
   - Feature flag/limit validation failures

---

### Struct: `ConfigValidator`

**Purpose**: Stateless validator for configuration

---

### Method: `ConfigValidator::validate_config()`

**Signature**: `pub fn validate_config(config: &AppConfig) -> Result<(), ConfigValidationError>`

**Purpose**: Main validation entry point

**Validation Steps**:

1. **Basic Validation** (via `validator` crate)
   ```rust
   config.validate()?;
   ```
   - Field-level constraints (email format, URL format, ranges)
   - Defined in struct annotations

2. **Security Validation**
   ```rust
   Self::validate_security_config(config)?;
   ```
   - Custom security rules

3. **Database Validation**
   ```rust
   Self::validate_database_config(config)?;
   ```
   - Connection pool constraints

4. **Feature Validation**
   ```rust
   Self::validate_feature_config(config)?;
   ```
   - Feature flag sanity checks

---

### Method: `validate_security_config()`

**Signature**: `fn validate_security_config(config: &AppConfig) -> Result<(), ConfigValidationError>`

**Purpose**: Validate security-critical settings

**Validations**:

#### 1. JWT Secret Strength
```rust
if security.jwt_secret.expose_secret().len() < 32 {
    return Err(ConfigValidationError::SecurityValidationFailed {
        message: "JWT secret must be at least 32 characters long".to_string(),
    });
}
```

**Rationale**:
- 32 characters = 256 bits minimum
- Prevents weak secrets that could be brute-forced
- Industry standard for symmetric keys

**Example Error**:
```
Security validation failed: JWT secret must be at least 32 characters long
```

---

#### 2. JWT Expiry Limit
```rust
if security.jwt_expiry_minutes > 60 {
    return Err(ConfigValidationError::SecurityValidationFailed {
        message: "JWT expiry should not exceed 60 minutes for security".to_string(),
    });
}
```

**Rationale**:
- Short-lived access tokens reduce exposure window
- 15-60 minutes is industry best practice
- Longer tokens increase risk if compromised

**Recommended Values**:
- **Development**: 15-30 minutes
- **Production**: 15 minutes
- **Maximum**: 60 minutes

---

#### 3. Password Minimum Length
```rust
if security.password_min_length < 8 {
    return Err(ConfigValidationError::SecurityValidationFailed {
        message: "Password minimum length should be at least 8 characters".to_string(),
    });
}
```

**Rationale**:
- NIST recommends minimum 8 characters
- Prevents weak passwords
- Balances security with usability

**Best Practices**:
- **Minimum**: 8 characters
- **Recommended**: 12 characters
- **Enterprise**: 14+ characters

---

### Method: `validate_database_config()`

**Signature**: `fn validate_database_config(config: &AppConfig) -> Result<(), ConfigValidationError>`

**Purpose**: Validate database connection settings

**Validations**:

#### 1. Connection Pool Consistency
```rust
if db.max_connections < db.min_connections {
    return Err(ConfigValidationError::DatabaseValidationFailed {
        message: "Max connections must be greater than or equal to min connections".to_string(),
    });
}
```

**Prevents**: Invalid pool configuration

---

#### 2. Connection Pool Upper Limit
```rust
if db.max_connections > 1000 {
    return Err(ConfigValidationError::DatabaseValidationFailed {
        message: "Max connections should not exceed 1000 for performance reasons".to_string(),
    });
}
```

**Rationale**:
- Database servers have connection limits
- Too many connections degrade performance
- 1000 is reasonable upper bound for most workloads

**Recommended Values**:
- **Development**: 5-10 connections
- **Production (small)**: 20-50 connections
- **Production (large)**: 100-200 connections
- **Maximum**: 1000 connections

---

### Method: `validate_feature_config()`

**Signature**: `fn validate_feature_config(config: &AppConfig) -> Result<(), ConfigValidationError>`

**Purpose**: Validate feature flags and limits

**Validations**:

#### Feature Limit Sanity Check
```rust
for (feature, limit) in &features.feature_limits {
    if *limit == 0 {
        return Err(ConfigValidationError::FeatureValidationFailed {
            message: format!("Feature limit for '{}' cannot be zero", feature),
        });
    }
}
```

**Prevents**: Zero limits that would block all usage

---

## Usage Examples

### Successful Validation

```rust
use auth_config::{ConfigLoader, ConfigValidator};

let loader = ConfigLoader::new("config", "production");
let config = loader.load()?;

// Validate configuration
ConfigValidator::validate_config(&config)?;

// Config is valid, proceed
println!("Configuration validated successfully");
```

---

### Handling Validation Errors

```rust
match ConfigValidator::validate_config(&config) {
    Ok(()) => {
        info!("Configuration validated successfully");
    }
    Err(ConfigValidationError::SecurityValidationFailed { message }) => {
        error!("Security validation failed: {}", message);
        std::process::exit(1);
    }
    Err(ConfigValidationError::DatabaseValidationFailed { message }) => {
        error!("Database validation failed: {}", message);
        std::process::exit(1);
    }
    Err(e) => {
        error!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }
}
```

---

### Integration with ConfigManager

```rust
impl ConfigManager {
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = self.loader.load()?;
        
        // Validate before applying
        ConfigValidator::validate_config(&new_config)
            .map_err(|e| anyhow::anyhow!("Invalid configuration: {}", e))?;
        
        // Apply validated config
        *self.current_config.write() = new_config;
        
        Ok(())
    }
}
```

---

## Security Best Practices

### 1. Fail Fast on Startup

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = ConfigLoader::new("config", "production").load()?;
    
    // Validate immediately
    ConfigValidator::validate_config(&config)?;
    
    // Only proceed if valid
    start_server(config).await
}
```

**Benefits**:
- Prevents running with insecure configuration
- Clear error messages for operators
- Faster debugging

---

### 2. Production Checklist

**Before Deployment**:
```rust
// ✅ JWT secret is strong (32+ characters)
assert!(config.security.jwt_secret.expose_secret().len() >= 32);

// ✅ Token expiry is reasonable (≤60 minutes)
assert!(config.security.jwt_expiry_minutes <= 60);

// ✅ Password policy is strong (≥8 characters)
assert!(config.security.password_min_length >= 8);

// ✅ Connection pool is configured
assert!(config.database.max_connections >= config.database.min_connections);
```

---

### 3. Environment-Specific Validation

```rust
pub fn validate_production_config(config: &AppConfig) -> Result<()> {
    ConfigValidator::validate_config(config)?;
    
    // Additional production-only checks
    if config.security.jwt_secret.expose_secret() == "change-me-in-production" {
        return Err(anyhow::anyhow!("Default JWT secret detected in production!"));
    }
    
    if !config.security.require_mfa {
        warn!("MFA not required in production - consider enabling");
    }
    
    Ok(())
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use auth_config::config::*;
    
    #[test]
    fn test_weak_jwt_secret_fails() {
        let mut config = AppConfig::default();
        config.security.jwt_secret = secrecy::Secret::new("short".to_string());
        
        let result = ConfigValidator::validate_config(&config);
        assert!(matches!(
            result,
            Err(ConfigValidationError::SecurityValidationFailed { .. })
        ));
    }
    
    #[test]
    fn test_excessive_jwt_expiry_fails() {
        let mut config = AppConfig::default();
        config.security.jwt_expiry_minutes = 120; // 2 hours
        
        let result = ConfigValidator::validate_config(&config);
        assert!(matches!(
            result,
            Err(ConfigValidationError::SecurityValidationFailed { .. })
        ));
    }
    
    #[test]
    fn test_invalid_connection_pool_fails() {
        let mut config = AppConfig::default();
        config.database.max_connections = 5;
        config.database.min_connections = 10; // Invalid!
        
        let result = ConfigValidator::validate_config(&config);
        assert!(matches!(
            result,
            Err(ConfigValidationError::DatabaseValidationFailed { .. })
        ));
    }
    
    #[test]
    fn test_valid_config_passes() {
        let config = AppConfig::default();
        assert!(ConfigValidator::validate_config(&config).is_ok());
    }
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `validator` | Field-level validation |
| `thiserror` | Error definitions |
| `secrecy` | Secret exposure for validation |

### Internal Dependencies

- [config.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-config/config.md) - Configuration structures

---

## Related Files

- [config.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-config/config.md) - Configuration structures
- [loader.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-config/loader.md) - Configuration loading
- [manager.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/manager.rs) - Configuration management

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 102  
**Security Level**: CRITICAL
