# config.rs

## File Metadata

**File Path**: `crates/auth-config/src/config.rs`  
**Crate**: `auth-config`  
**Module**: `config`  
**Layer**: Infrastructure  
**Security-Critical**: ✅ **YES** - Contains sensitive configuration including secrets

## Purpose

Defines the complete configuration structure for the SSO platform with strongly-typed settings for server, database, security, features, logging, and external services.

### Problem It Solves

- Type-safe configuration management
- Validation of configuration values
- Secure handling of secrets (passwords, API keys)
- Environment-specific settings
- Feature flags and tenant overrides

---

## Detailed Code Breakdown

### Struct: `AppConfig`

**Purpose**: Root configuration structure

**Fields**:
- `server`: Server configuration (port, host, workers)
- `database`: Database connection settings
- `security`: Security policies and secrets
- `features`: Feature flags and limits
- `logging`: Logging configuration
- `external_services`: SMTP, SMS, Redis settings

**Validation**: `#[derive(Validate)]` enables automatic validation

---

### Struct: `ServerConfig`

**Fields**:

| Field | Type | Description | Validation | Default |
|-------|------|-------------|------------|---------|
| `port` | `u16` | HTTP server port | 1-65535 | 8081 |
| `host` | `String` | Bind address | - | "0.0.0.0" |
| `workers` | `Option<usize>` | Worker threads | - | None (auto) |
| `max_connections` | `Option<u32>` | Max concurrent connections | - | 1000 |
| `timeout_seconds` | `Option<u64>` | Request timeout | - | 30 |

**Examples**:
```toml
[server]
port = 8080
host = "0.0.0.0"
workers = 4
max_connections = 1000
timeout_seconds = 30
```

---

### Struct: `DatabaseConfig`

**Fields**:

| Field | Type | Description | Security |
|-------|------|-------------|----------|
| `mysql_url` | `Secret<String>` | MySQL connection string | **CRITICAL**: Never logged |
| `sqlite_url` | `Option<String>` | SQLite path (testing) | - |
| `max_connections` | `u32` | Connection pool max | Default: 10 |
| `min_connections` | `u32` | Connection pool min | Default: 1 |
| `connection_timeout` | `u64` | Connection timeout (seconds) | Default: 30 |
| `idle_timeout` | `u64` | Idle connection timeout | Default: 600 |
| `max_lifetime` | `u64` | Max connection lifetime | Default: 3600 |

**Secret Protection**:
```rust
#[serde(skip_serializing)]
pub mysql_url: secrecy::Secret<String>
```
- **Never serialized** to logs or JSON
- **Redacted** in debug output
- **Requires explicit** `.expose_secret()` to access

**Example**:
```toml
[database]
mysql_url = "mysql://user:password@localhost:3306/auth_db"
max_connections = 20
min_connections = 5
connection_timeout = 30
idle_timeout = 600
max_lifetime = 3600
```

---

### Struct: `SecurityConfig`

**Fields**:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `jwt_secret` | `Secret<String>` | JWT signing key | **MUST CHANGE** |
| `jwt_expiry_minutes` | `u32` | Access token TTL | 15 minutes |
| `refresh_token_expiry_days` | `u32` | Refresh token TTL | 30 days |
| `password_min_length` | `u8` | Minimum password length | 8 characters |
| `max_login_attempts` | `u32` | Failed login threshold | 5 attempts |
| `lockout_duration_minutes` | `u32` | Account lockout duration | 15 minutes |
| `require_mfa` | `bool` | Enforce MFA globally | false |
| `allowed_origins` | `Vec<String>` | CORS allowed origins | ["http://localhost:3000"] |

**Critical Security Settings**:

1. **JWT Secret**:
   ```rust
   #[serde(skip_serializing)]
   pub jwt_secret: secrecy::Secret<String>
   ```
   - **MUST** be changed in production
   - **Minimum** 32 bytes of randomness
   - **Rotation**: Plan for key rotation

2. **Token Expiry**:
   - Access: 15 minutes (short-lived)
   - Refresh: 30 days (long-lived)

3. **Account Lockout**:
   - 5 failed attempts → 15-minute lockout
   - Prevents brute-force attacks

**Example**:
```toml
[security]
jwt_secret = "your-256-bit-secret-key-here"
jwt_expiry_minutes = 15
refresh_token_expiry_days = 30
password_min_length = 12
max_login_attempts = 5
lockout_duration_minutes = 30
require_mfa = true
allowed_origins = ["https://app.example.com"]
```

---

### Struct: `FeatureConfig`

**Purpose**: Feature flags and tenant-specific overrides

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `enabled_features` | `HashMap<String, bool>` | Global feature flags |
| `feature_limits` | `HashMap<String, u64>` | Resource quotas |
| `tenant_overrides` | `HashMap<String, HashMap<String, Value>>` | Per-tenant settings |

**Use Cases**:

1. **Feature Flags**:
   ```toml
   [features.enabled_features]
   oauth = true
   saml = true
   scim = false
   passwordless = true
   ```

2. **Resource Limits**:
   ```toml
   [features.feature_limits]
   max_users_per_tenant = 1000
   max_api_calls_per_hour = 10000
   max_sessions_per_user = 5
   ```

3. **Tenant Overrides**:
   ```toml
   [features.tenant_overrides.acme-corp]
   max_users_per_tenant = 10000
   require_mfa = true
   ```

---

### Struct: `LoggingConfig`

**Fields**:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `level` | `String` | Log level | "info" |
| `format` | `String` | Log format | "json" |
| `output` | `String` | Output destination | "stdout" |
| `structured` | `bool` | Structured logging | true |

**Log Levels**: `trace`, `debug`, `info`, `warn`, `error`

**Formats**:
- `json`: Structured JSON (production)
- `pretty`: Human-readable (development)

**Example**:
```toml
[logging]
level = "info"
format = "json"
output = "stdout"
structured = true
```

---

### Struct: `ExternalServicesConfig`

**Purpose**: Configuration for external integrations

**Fields**:
- `smtp`: Email service configuration
- `sms`: SMS provider configuration
- `redis`: Redis cache configuration

---

### Struct: `SmtpConfig`

**Purpose**: Email service configuration

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `host` | `String` | SMTP server hostname |
| `port` | `u16` | SMTP port (25, 587, 465) |
| `username` | `String` | SMTP username |
| `password` | `Secret<String>` | SMTP password (protected) |
| `from_address` | `String` | Default sender address |

**Example**:
```toml
[external_services.smtp]
host = "smtp.gmail.com"
port = 587
username = "noreply@example.com"
password = "app-specific-password"
from_address = "noreply@example.com"
```

---

### Struct: `SmsConfig`

**Purpose**: SMS provider configuration

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `provider` | `String` | Provider name (twilio, nexmo) |
| `api_key` | `Secret<String>` | API key (protected) |
| `from_number` | `String` | Sender phone number |

**Example**:
```toml
[external_services.sms]
provider = "twilio"
api_key = "your-twilio-api-key"
from_number = "+1234567890"
```

---

### Struct: `RedisConfig`

**Purpose**: Redis cache configuration

**Fields**:

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `url` | `String` | Redis connection URL | - |
| `max_connections` | `u32` | Connection pool size | - |
| `timeout_seconds` | `u64` | Operation timeout | - |

**Example**:
```toml
[external_services.redis]
url = "redis://localhost:6379"
max_connections = 10
timeout_seconds = 5
```

---

## Default Configuration

### Implementation

```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8081,
                host: "0.0.0.0".to_string(),
                workers: None,
                max_connections: Some(1000),
                timeout_seconds: Some(30),
            },
            database: DatabaseConfig {
                mysql_url: Secret::new("mysql://localhost/auth".to_string()),
                sqlite_url: Some(":memory:".to_string()),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 3600,
            },
            security: SecurityConfig {
                jwt_secret: Secret::new("change-me-in-production".to_string()),
                jwt_expiry_minutes: 15,
                refresh_token_expiry_days: 30,
                password_min_length: 8,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
                require_mfa: false,
                allowed_origins: vec!["http://localhost:3000".to_string()],
            },
            features: FeatureConfig {
                enabled_features: HashMap::new(),
                feature_limits: HashMap::new(),
                tenant_overrides: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                output: "stdout".to_string(),
                structured: true,
            },
            external_services: ExternalServicesConfig {
                smtp: None,
                sms: None,
                redis: None,
            },
        }
    }
}
```

**Usage**:
```rust
let config = AppConfig::default();
```

---

## Security Considerations

### 1. Secret Protection

**secrecy Crate**:
```rust
use secrecy::Secret;

#[serde(skip_serializing)]
pub jwt_secret: Secret<String>
```

**Benefits**:
- Never accidentally logged
- Explicit access required
- Redacted in debug output

**Access**:
```rust
let secret_value = config.security.jwt_secret.expose_secret();
```

### 2. Configuration Validation

**Validator Integration**:
```rust
#[derive(Validate)]
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
}
```

**Usage**:
```rust
config.validate()?;
```

### 3. Production Checklist

- [ ] Change `jwt_secret` to random 256-bit value
- [ ] Set `mysql_url` to production database
- [ ] Configure `allowed_origins` for CORS
- [ ] Enable `require_mfa` if needed
- [ ] Set appropriate `max_connections`
- [ ] Configure SMTP for email notifications
- [ ] Review `lockout_duration_minutes`

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `serde` | Serialization |
| `secrecy` | Secret protection |
| `validator` | Validation |

---

## Related Files

- [loader.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/loader.rs) - Configuration loading
- [manager.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/manager.rs) - Configuration management
- [main.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/src/main.md) - Configuration usage

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 151  
**Security Level**: CRITICAL
