# loader.rs

## File Metadata

**File Path**: `crates/auth-config/src/loader.rs`  
**Crate**: `auth-config`  
**Module**: `loader`  
**Layer**: Infrastructure  
**Security-Critical**: ⚠️ **MEDIUM** - Loads configuration from multiple sources

## Purpose

Loads configuration from multiple sources with defined precedence: environment variables, local files, environment-specific files, and default files.

### Problem It Solves

- Multi-source configuration loading
- Environment-specific settings (development, staging, production)
- Local overrides for development (gitignored)
- Environment variable overrides for deployment

---

## Detailed Code Breakdown

### Struct: `ConfigLoader`

**Fields**:
- `config_dir`: Directory containing config files (e.g., "config")
- `environment`: Environment name (e.g., "development", "production")

---

### Method: `ConfigLoader::new()`

**Signature**: `pub fn new(config_dir, environment) -> Self`

**Purpose**: Create new configuration loader

**Usage**:
```rust
let loader = ConfigLoader::new("config", "production");
```

---

### Method: `ConfigLoader::load()`

**Signature**: `pub fn load(&self) -> Result<AppConfig, ConfigError>`

**Purpose**: Load configuration from all sources with precedence

**Loading Order** (lowest to highest precedence):

1. **Default Configuration** (`config/default.toml`)
   ```rust
   config = config.add_source(File::with_name(&format!(
       "{}/default",
       self.config_dir
   )).required(false));
   ```
   - **Purpose**: Base configuration
   - **Required**: No (optional)
   - **Example**: `config/default.toml`

2. **Environment-Specific** (`config/{environment}.toml`)
   ```rust
   config = config.add_source(File::with_name(&format!(
       "{}/{}",
       self.config_dir, self.environment
   )).required(false));
   ```
   - **Purpose**: Environment overrides
   - **Required**: No (optional)
   - **Examples**: 
     - `config/development.toml`
     - `config/staging.toml`
     - `config/production.toml`

3. **Local Configuration** (`config/local.toml`)
   ```rust
   config = config.add_source(File::with_name(&format!(
       "{}/local",
       self.config_dir
   )).required(false));
   ```
   - **Purpose**: Developer-specific overrides
   - **Required**: No (optional)
   - **Gitignored**: Yes (never committed)
   - **Example**: `config/local.toml`

4. **Environment Variables** (highest precedence)
   ```rust
   config = config.add_source(
       Environment::with_prefix("AUTH")
           .separator("__")
           .try_parsing(true)
   );
   ```
   - **Purpose**: Deployment-time overrides
   - **Prefix**: `AUTH`
   - **Separator**: `__` (double underscore)
   - **Examples**:
     - `AUTH__SERVER__PORT=8080`
     - `AUTH__DATABASE__MAX_CONNECTIONS=20`
     - `AUTH__SECURITY__JWT_SECRET=secret-key`

---

## Configuration Precedence

### Example Scenario

**Files**:
```toml
# config/default.toml
[server]
port = 8081
host = "127.0.0.1"

# config/production.toml
[server]
host = "0.0.0.0"

# config/local.toml (gitignored)
[server]
port = 3000
```

**Environment Variables**:
```bash
AUTH__SERVER__PORT=8080
```

**Result**:
```rust
AppConfig {
    server: ServerConfig {
        port: 8080,        // From environment variable (highest)
        host: "0.0.0.0",   // From production.toml
    }
}
```

**Precedence Order**:
1. Environment variables: `port = 8080` ✅ **WINS**
2. Local file: `port = 3000`
3. Environment file: `host = "0.0.0.0"` ✅ **WINS**
4. Default file: `port = 8081`, `host = "127.0.0.1"`

---

## Environment Variable Mapping

### Naming Convention

**Format**: `AUTH__{SECTION}__{KEY}`

**Examples**:

| Config Path | Environment Variable |
|-------------|---------------------|
| `server.port` | `AUTH__SERVER__PORT` |
| `database.max_connections` | `AUTH__DATABASE__MAX_CONNECTIONS` |
| `security.jwt_secret` | `AUTH__SECURITY__JWT_SECRET` |
| `logging.level` | `AUTH__LOGGING__LEVEL` |

### Nested Structures

**TOML**:
```toml
[external_services.smtp]
host = "smtp.gmail.com"
port = 587
```

**Environment Variables**:
```bash
AUTH__EXTERNAL_SERVICES__SMTP__HOST=smtp.gmail.com
AUTH__EXTERNAL_SERVICES__SMTP__PORT=587
```

---

### Method: `ConfigLoader::load_from_file()`

**Signature**: `pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<AppConfig>`

**Purpose**: Load configuration from a single file

**Usage**:
```rust
let config = ConfigLoader::load_from_file("config/production.toml")?;
```

**Use Cases**:
- Testing with specific config
- Loading from custom location
- Configuration validation

---

### Method: `ConfigLoader::load_from_env()`

**Signature**: `pub fn load_from_env() -> Result<AppConfig>`

**Purpose**: Load configuration exclusively from environment variables

**Usage**:
```rust
let config = ConfigLoader::load_from_env()?;
```

**Use Cases**:
- Containerized deployments (Docker, Kubernetes)
- 12-factor app compliance
- No filesystem access

**Example**:
```bash
# Set all required config via env vars
export AUTH__SERVER__PORT=8080
export AUTH__SERVER__HOST=0.0.0.0
export AUTH__DATABASE__MYSQL_URL=mysql://user:pass@db:3306/auth
export AUTH__SECURITY__JWT_SECRET=your-secret-key

# Run application
./auth-platform
```

---

## Configuration Files

### File Structure

```
project/
├── config/
│   ├── default.toml          # Base configuration
│   ├── development.toml      # Development overrides
│   ├── staging.toml          # Staging overrides
│   ├── production.toml       # Production overrides
│   └── local.toml            # Local overrides (gitignored)
└── .gitignore
```

### .gitignore

```gitignore
# Ignore local configuration
config/local.toml

# Ignore environment files with secrets
.env
.env.local
```

---

## Example Configurations

### default.toml

```toml
[server]
port = 8081
host = "127.0.0.1"
workers = 4

[database]
max_connections = 10
min_connections = 2
connection_timeout = 30

[security]
jwt_expiry_minutes = 15
refresh_token_expiry_days = 30
password_min_length = 8
max_login_attempts = 5
lockout_duration_minutes = 15
require_mfa = false

[logging]
level = "info"
format = "json"
output = "stdout"
structured = true
```

### production.toml

```toml
[server]
port = 8080
host = "0.0.0.0"
workers = 8
max_connections = 2000

[database]
max_connections = 50
min_connections = 10

[security]
require_mfa = true
allowed_origins = ["https://app.example.com"]

[logging]
level = "warn"
```

### local.toml (development)

```toml
[server]
port = 3000

[database]
sqlite_url = "sqlite::memory:"

[logging]
level = "debug"
format = "pretty"
```

---

## Deployment Patterns

### Pattern 1: File-Based (Traditional)

**Setup**:
```bash
# Copy environment-specific config
cp config/production.toml /etc/auth/config.toml

# Run with environment
AUTH_ENVIRONMENT=production ./auth-platform
```

**Pros**:
- Simple deployment
- Easy to review entire config

**Cons**:
- Secrets in files
- File management complexity

---

### Pattern 2: Environment Variables (12-Factor)

**Setup**:
```bash
# Kubernetes ConfigMap
kubectl create configmap auth-config \
  --from-literal=AUTH__SERVER__PORT=8080 \
  --from-literal=AUTH__DATABASE__MAX_CONNECTIONS=50

# Kubernetes Secret
kubectl create secret generic auth-secrets \
  --from-literal=AUTH__DATABASE__MYSQL_URL=mysql://... \
  --from-literal=AUTH__SECURITY__JWT_SECRET=...
```

**Pros**:
- No secrets in files
- Cloud-native
- Easy updates

**Cons**:
- Harder to review
- Many environment variables

---

### Pattern 3: Hybrid (Recommended)

**Setup**:
```bash
# Base config from file
cp config/production.toml /etc/auth/config.toml

# Secrets from environment
export AUTH__DATABASE__MYSQL_URL=mysql://...
export AUTH__SECURITY__JWT_SECRET=...

# Run
./auth-platform
```

**Pros**:
- Secrets not in files
- Easy to review non-secret config
- Flexible

---

## Error Handling

### Configuration Errors

```rust
match loader.load() {
    Ok(config) => {
        // Validate
        config.validate()?;
        // Use config
    }
    Err(ConfigError::NotFound(path)) => {
        eprintln!("Config file not found: {}", path);
    }
    Err(ConfigError::Parse(msg)) => {
        eprintln!("Config parse error: {}", msg);
    }
    Err(e) => {
        eprintln!("Config error: {}", e);
    }
}
```

---

## Testing

### Unit Tests

```rust
#[test]
fn test_load_from_file() {
    let config = ConfigLoader::load_from_file("config/test.toml").unwrap();
    assert_eq!(config.server.port, 8081);
}

#[test]
fn test_env_override() {
    std::env::set_var("AUTH__SERVER__PORT", "9000");
    let loader = ConfigLoader::new("config", "test");
    let config = loader.load().unwrap();
    assert_eq!(config.server.port, 9000);
}
```

---

## Security Considerations

### 1. Secret Management

**Bad** (secrets in files):
```toml
# config/production.toml
[security]
jwt_secret = "my-secret-key"  # DON'T: Committed to git
```

**Good** (secrets in environment):
```bash
# Environment variable
export AUTH__SECURITY__JWT_SECRET=my-secret-key

# Or use secret management
export AUTH__SECURITY__JWT_SECRET=$(vault read -field=value secret/jwt)
```

### 2. File Permissions

```bash
# Restrict config file access
chmod 600 /etc/auth/config.toml
chown auth:auth /etc/auth/config.toml
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `config` | Configuration loading |
| `anyhow` | Error handling |

### Internal Dependencies

- [config.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-config/config.md) - Configuration structures

---

## Related Files

- [config.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-config/config.md) - Configuration structures
- [manager.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/manager.rs) - Configuration management
- [main.md](file:///c:/Users/Victo/Downloads/sso/docs/code/src/main.md) - Configuration usage

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 72  
**Security Level**: MEDIUM
