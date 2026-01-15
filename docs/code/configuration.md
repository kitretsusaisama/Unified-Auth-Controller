# Configuration & Runtime Behavior

This document provides comprehensive documentation for all configuration options, environment variables, and runtime behavior of the Enterprise SSO Platform.

## Table of Contents

1. [Configuration Sources](#configuration-sources)
2. [Environment Variables](#environment-variables)
3. [Configuration Files](#configuration-files)
4. [Startup Sequence](#startup-sequence)
5. [Runtime Modes](#runtime-modes)
6. [Feature Flags](#feature-flags)

---

## Configuration Sources

The platform supports multiple configuration sources with the following precedence (highest to lowest):

```
1. Environment Variables (AUTH__*)
2. Local Configuration (config/local.toml)
3. Environment-Specific (config/development.toml, config/production.toml)
4. Default Configuration (config/default.toml)
```

### Configuration Loading

**File**: [auth-config/src/loader.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/loader.rs)

```rust
pub struct ConfigLoader {
    config_dir: PathBuf,
    environment: String,
}

impl ConfigLoader {
    pub fn new(config_dir: &str, environment: &str) -> Self {
        Self {
            config_dir: PathBuf::from(config_dir),
            environment: environment.to_string(),
        }
    }
    
    pub fn load(&self) -> Result<Config, ConfigError> {
        let mut builder = config::Config::builder()
            // 1. Load default.toml
            .add_source(config::File::from(self.config_dir.join("default.toml")))
            // 2. Load environment-specific (development.toml, production.toml)
            .add_source(config::File::from(
                self.config_dir.join(format!("{}.toml", self.environment))
            ).required(false))
            // 3. Load local.toml (gitignored)
            .add_source(config::File::from(self.config_dir.join("local.toml")).required(false))
            // 4. Load environment variables with AUTH__ prefix
            .add_source(config::Environment::with_prefix("AUTH").separator("__"));
        
        builder.build()?.try_deserialize()
    }
}
```

---

## Environment Variables

All configuration can be overridden using environment variables with the `AUTH__` prefix.

### Naming Convention

**Format**: `AUTH__SECTION__KEY=value`

**Example**:
```bash
AUTH__SERVER__PORT=8080
AUTH__DATABASE__MYSQL_URL=mysql://user:pass@localhost/db
AUTH__SECURITY__JWT_SECRET=your-secret-key
```

### Core Environment Variables

#### Database Configuration

##### `DATABASE_URL` (Required)
**Description**: Primary database connection string  
**Format**: `mysql://user:password@host:port/database` or `sqlite:./path/to/db.sqlite`  
**Example**: `mysql://auth_user:secure_password@localhost:3306/auth_platform`  
**Used By**: Main application, migrations

##### `AUTH__DATABASE__MAX_CONNECTIONS`
**Description**: Maximum database connection pool size  
**Type**: Integer  
**Default**: `10`  
**Range**: `1-100`  
**Example**: `AUTH__DATABASE__MAX_CONNECTIONS=20`

##### `AUTH__DATABASE__MIN_CONNECTIONS`
**Description**: Minimum database connection pool size  
**Type**: Integer  
**Default**: `2`  
**Example**: `AUTH__DATABASE__MIN_CONNECTIONS=5`

##### `AUTH__DATABASE__CONNECT_TIMEOUT`
**Description**: Database connection timeout in seconds  
**Type**: Integer  
**Default**: `30`  
**Example**: `AUTH__DATABASE__CONNECT_TIMEOUT=60`

---

#### Server Configuration

##### `AUTH__SERVER__HOST`
**Description**: HTTP server bind address  
**Type**: String  
**Default**: `127.0.0.1`  
**Production**: `0.0.0.0` (all interfaces)  
**Example**: `AUTH__SERVER__HOST=0.0.0.0`

##### `AUTH__SERVER__PORT`
**Description**: HTTP server port  
**Type**: Integer  
**Default**: `8080`  
**Range**: `1024-65535`  
**Example**: `AUTH__SERVER__PORT=3000`

##### `AUTH__SERVER__WORKERS`
**Description**: Number of worker threads  
**Type**: Integer  
**Default**: Number of CPU cores  
**Example**: `AUTH__SERVER__WORKERS=4`

##### `AUTH__SERVER__REQUEST_TIMEOUT`
**Description**: HTTP request timeout in seconds  
**Type**: Integer  
**Default**: `30`  
**Example**: `AUTH__SERVER__REQUEST_TIMEOUT=60`

---

#### Security Configuration

##### `AUTH__SECURITY__JWT_SECRET` (Required for HS256)
**Description**: JWT signing secret (for HS256 algorithm)  
**Type**: String (Base64 encoded)  
**Security**: **CRITICAL - Never commit to version control**  
**Generation**: `openssl rand -base64 32`  
**Example**: `AUTH__SECURITY__JWT_SECRET=your-base64-encoded-secret`

##### `AUTH__SECURITY__JWT_PRIVATE_KEY_PATH` (Required for RS256)
**Description**: Path to RSA private key for JWT signing  
**Type**: File path  
**Format**: PEM-encoded RSA private key (2048-bit minimum)  
**Example**: `AUTH__SECURITY__JWT_PRIVATE_KEY_PATH=/secrets/jwt_private.pem`  
**Generation**:
```bash
openssl genrsa -out jwt_private.pem 2048
openssl rsa -in jwt_private.pem -pubout -out jwt_public.pem
```

##### `AUTH__SECURITY__JWT_PUBLIC_KEY_PATH` (Required for RS256)
**Description**: Path to RSA public key for JWT verification  
**Type**: File path  
**Example**: `AUTH__SECURITY__JWT_PUBLIC_KEY_PATH=/secrets/jwt_public.pem`

##### `AUTH__SECURITY__JWT_ALGORITHM`
**Description**: JWT signing algorithm  
**Type**: Enum  
**Values**: `RS256` (recommended), `HS256`  
**Default**: `RS256`  
**Example**: `AUTH__SECURITY__JWT_ALGORITHM=RS256`

##### `AUTH__SECURITY__ACCESS_TOKEN_EXPIRATION`
**Description**: Access token expiration time  
**Type**: Duration string  
**Default**: `15m` (15 minutes)  
**Format**: `{number}{unit}` where unit is `s`, `m`, `h`, `d`  
**Example**: `AUTH__SECURITY__ACCESS_TOKEN_EXPIRATION=30m`

##### `AUTH__SECURITY__REFRESH_TOKEN_EXPIRATION`
**Description**: Refresh token expiration time  
**Type**: Duration string  
**Default**: `7d` (7 days)  
**Example**: `AUTH__SECURITY__REFRESH_TOKEN_EXPIRATION=30d`

##### `AUTH__SECURITY__PASSWORD_MIN_LENGTH`
**Description**: Minimum password length  
**Type**: Integer  
**Default**: `8`  
**Range**: `8-128`  
**Example**: `AUTH__SECURITY__PASSWORD_MIN_LENGTH=12`

##### `AUTH__SECURITY__PASSWORD_REQUIRE_UPPERCASE`
**Description**: Require uppercase letters in passwords  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__SECURITY__PASSWORD_REQUIRE_UPPERCASE=false`

##### `AUTH__SECURITY__PASSWORD_REQUIRE_LOWERCASE`
**Description**: Require lowercase letters in passwords  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__SECURITY__PASSWORD_REQUIRE_LOWERCASE=true`

##### `AUTH__SECURITY__PASSWORD_REQUIRE_NUMBERS`
**Description**: Require numbers in passwords  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__SECURITY__PASSWORD_REQUIRE_NUMBERS=true`

##### `AUTH__SECURITY__PASSWORD_REQUIRE_SPECIAL`
**Description**: Require special characters in passwords  
**Type**: Boolean  
**Default**: `false`  
**Example**: `AUTH__SECURITY__PASSWORD_REQUIRE_SPECIAL=true`

##### `AUTH__SECURITY__MAX_FAILED_ATTEMPTS`
**Description**: Maximum failed login attempts before lockout  
**Type**: Integer  
**Default**: `5`  
**Range**: `3-10`  
**Example**: `AUTH__SECURITY__MAX_FAILED_ATTEMPTS=3`

##### `AUTH__SECURITY__LOCKOUT_DURATION`
**Description**: Account lockout duration after max failed attempts  
**Type**: Duration string  
**Default**: `30m`  
**Example**: `AUTH__SECURITY__LOCKOUT_DURATION=1h`

---

#### Rate Limiting Configuration

##### `AUTH__RATE_LIMIT__LOGIN_ATTEMPTS`
**Description**: Maximum login attempts per IP per minute  
**Type**: Integer  
**Default**: `5`  
**Example**: `AUTH__RATE_LIMIT__LOGIN_ATTEMPTS=10`

##### `AUTH__RATE_LIMIT__REGISTRATION_ATTEMPTS`
**Description**: Maximum registration attempts per IP per hour  
**Type**: Integer  
**Default**: `3`  
**Example**: `AUTH__RATE_LIMIT__REGISTRATION_ATTEMPTS=5`

##### `AUTH__RATE_LIMIT__API_REQUESTS`
**Description**: Maximum API requests per user per minute  
**Type**: Integer  
**Default**: `100`  
**Example**: `AUTH__RATE_LIMIT__API_REQUESTS=200`

---

#### Cache Configuration

##### `AUTH__CACHE__REDIS_URL`
**Description**: Redis connection string for distributed caching  
**Type**: String  
**Format**: `redis://[user:password@]host:port[/database]`  
**Example**: `AUTH__CACHE__REDIS_URL=redis://:password@localhost:6379/0`  
**Optional**: If not set, uses in-memory cache only

##### `AUTH__CACHE__REDIS_POOL_SIZE`
**Description**: Redis connection pool size  
**Type**: Integer  
**Default**: `10`  
**Example**: `AUTH__CACHE__REDIS_POOL_SIZE=20`

##### `AUTH__CACHE__DEFAULT_TTL`
**Description**: Default cache TTL in seconds  
**Type**: Integer  
**Default**: `300` (5 minutes)  
**Example**: `AUTH__CACHE__DEFAULT_TTL=600`

##### `AUTH__CACHE__ENABLED`
**Description**: Enable/disable caching  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__CACHE__ENABLED=false`

---

#### Logging Configuration

##### `RUST_LOG`
**Description**: Logging level and filters  
**Type**: String  
**Format**: `target=level,target=level,...`  
**Default**: `info`  
**Example**: `RUST_LOG=auth_platform=debug,auth_api=debug,sqlx=warn`

**Levels**: `trace`, `debug`, `info`, `warn`, `error`

##### `AUTH__LOGGING__FORMAT`
**Description**: Log output format  
**Type**: Enum  
**Values**: `json`, `pretty`, `compact`  
**Default**: `json` (production), `pretty` (development)  
**Example**: `AUTH__LOGGING__FORMAT=json`

##### `AUTH__LOGGING__LEVEL`
**Description**: Global logging level  
**Type**: Enum  
**Values**: `trace`, `debug`, `info`, `warn`, `error`  
**Default**: `info`  
**Example**: `AUTH__LOGGING__LEVEL=debug`

---

#### Telemetry Configuration

##### `AUTH__TELEMETRY__ENABLED`
**Description**: Enable/disable telemetry  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__TELEMETRY__ENABLED=false`

##### `AUTH__TELEMETRY__PROMETHEUS_PORT`
**Description**: Prometheus metrics endpoint port  
**Type**: Integer  
**Default**: `9090`  
**Example**: `AUTH__TELEMETRY__PROMETHEUS_PORT=9091`

##### `AUTH__TELEMETRY__TRACING_ENDPOINT`
**Description**: OpenTelemetry tracing endpoint  
**Type**: String  
**Format**: `http://host:port`  
**Example**: `AUTH__TELEMETRY__TRACING_ENDPOINT=http://jaeger:14268/api/traces`

---

#### Audit Configuration

##### `AUTH__AUDIT__ENABLED`
**Description**: Enable/disable audit logging  
**Type**: Boolean  
**Default**: `true`  
**Example**: `AUTH__AUDIT__ENABLED=false`

##### `AUTH__AUDIT__RETENTION_DAYS`
**Description**: Audit log retention period in days  
**Type**: Integer  
**Default**: `365` (1 year)  
**Example**: `AUTH__AUDIT__RETENTION_DAYS=2555` (7 years for compliance)

---

## Configuration Files

### Default Configuration

**File**: `config/default.toml`  
**Purpose**: Base configuration for all environments  
**Committed**: Yes

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4
request_timeout = 30

[database]
max_connections = 10
min_connections = 2
connect_timeout = 30

[security]
jwt_algorithm = "RS256"
access_token_expiration = "15m"
refresh_token_expiration = "7d"
password_min_length = 8
password_require_uppercase = true
password_require_lowercase = true
password_require_numbers = true
password_require_special = false
max_failed_attempts = 5
lockout_duration = "30m"

[rate_limit]
login_attempts = 5
registration_attempts = 3
api_requests = 100

[cache]
enabled = true
default_ttl = 300

[logging]
level = "info"
format = "json"

[telemetry]
enabled = true
prometheus_port = 9090

[audit]
enabled = true
retention_days = 365
```

---

### Development Configuration

**File**: `config/development.toml`  
**Purpose**: Development-specific overrides  
**Committed**: Yes

```toml
[server]
host = "127.0.0.1"
port = 8081

[logging]
level = "debug"
format = "pretty"

[security]
# More lenient for development
password_min_length = 6
max_failed_attempts = 10

[cache]
# Disable Redis in development (use in-memory only)
enabled = false
```

---

### Production Configuration

**File**: `config/production.toml`  
**Purpose**: Production-specific settings  
**Committed**: Yes

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 8
request_timeout = 60

[logging]
level = "info"
format = "json"

[security]
# Stricter for production
password_min_length = 12
password_require_special = true
max_failed_attempts = 3
lockout_duration = "1h"

[cache]
enabled = true

[telemetry]
enabled = true

[audit]
enabled = true
retention_days = 2555  # 7 years for compliance
```

---

### Local Configuration

**File**: `config/local.toml`  
**Purpose**: Local developer overrides  
**Committed**: No (gitignored)

```toml
# Example local.toml (not committed)

[server]
port = 3000

[database]
# Override with local database
# (DATABASE_URL env var takes precedence)

[logging]
level = "trace"
format = "pretty"
```

---

## Startup Sequence

### Application Initialization

**File**: [src/main.rs](file:///c:/Users/Victo/Downloads/sso/src/main.rs)

#### Step 1: Environment Loading
```rust
dotenvy::dotenv().ok();
```
- Loads `.env` file (development only)
- Sets environment variables

#### Step 2: Logging Initialization
```rust
tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "auth_platform=debug,auth_api=debug,tower_http=debug".into()))
    .with(tracing_subscriber::fmt::layer())
    .init();
```
- Initializes structured logging
- Configures log levels from `RUST_LOG`

#### Step 3: Configuration Loading
```rust
let environment = std::env::var("AUTH_ENVIRONMENT")
    .unwrap_or_else(|_| "development".to_string());
let config_loader = ConfigLoader::new("config", &environment);
let config_manager = ConfigManager::new(config_loader)?;
let config = config_manager.get_config();
```
- Determines environment (development/production)
- Loads configuration from multiple sources
- Validates configuration

#### Step 4: Database Connection
```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
let pool = MySqlPoolOptions::new()
    .max_connections(config.database.max_connections)
    .connect(&database_url)
    .await?;
```
- Connects to MySQL/SQLite
- Creates connection pool
- Tests connectivity

#### Step 5: Database Migrations
```rust
sqlx::migrate!()
    .run(&pool)
    .await?;
```
- Runs pending migrations
- Ensures schema is up-to-date

#### Step 6: Repository Initialization
```rust
let role_repo = Arc::new(RoleRepository::new(pool.clone()));
let session_repo = Arc::new(SessionRepository::new(pool.clone()));
let subscription_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
let user_repo = Arc::new(UserRepository::new(pool.clone()));
```
- Creates repository instances
- Wraps in Arc for shared ownership

#### Step 7: Service Initialization
```rust
let role_service = Arc::new(RoleService::new(role_repo));
let risk_engine = Arc::new(RiskEngine::new());
let session_service = Arc::new(SessionService::new(session_repo, risk_engine));
let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));
let token_service = Arc::new(TokenEngine::new().await?);
let identity_service = Arc::new(IdentityService::new(user_repo, token_service));
```
- Wires together services
- Dependency injection

#### Step 8: Application State
```rust
let app_state = AppState {
    db: pool,
    role_service,
    session_service,
    subscription_service,
    identity_service,
};
```
- Creates shared application state
- Passed to all HTTP handlers

#### Step 9: Router Initialization
```rust
let app = auth_api::app(app_state);
```
- Builds Axum router
- Configures middleware
- Registers routes

#### Step 10: Server Startup
```rust
let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
let listener = tokio::net::TcpListener::bind(addr).await?;
axum::serve(listener, app).await?;
```
- Binds to configured address
- Starts HTTP server
- Begins accepting connections

---

## Runtime Modes

### Development Mode

**Activation**: `AUTH_ENVIRONMENT=development`

**Characteristics**:
- SQLite database (default)
- Verbose logging (debug level)
- Pretty-printed logs
- Hot reload (with cargo-watch)
- Relaxed security settings
- In-memory caching only
- Swagger UI enabled

**Use Cases**:
- Local development
- Testing
- Debugging

---

### Production Mode

**Activation**: `AUTH_ENVIRONMENT=production`

**Characteristics**:
- MySQL database (required)
- Structured JSON logs
- Info-level logging
- Optimized builds (release mode)
- Strict security settings
- Redis caching
- Swagger UI disabled
- TLS required

**Use Cases**:
- Production deployment
- Staging environment
- Performance testing

---

## Feature Flags

### Compile-Time Features

**File**: `Cargo.toml`

Currently, the platform uses runtime configuration instead of compile-time features. Future feature flags may include:

- `graphql` - Enable GraphQL API (auth-extension)
- `saml` - Enable SAML support (auth-protocols)
- `scim` - Enable SCIM support (auth-protocols)
- `redis` - Enable Redis caching (auth-cache)
- `metrics` - Enable Prometheus metrics (auth-telemetry)

---

## Configuration Validation

**File**: [auth-config/src/validation.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-config/src/validation.rs)

### Validation Rules

1. **Server Port**: Must be between 1024-65535
2. **Database Connections**: max_connections >= min_connections
3. **JWT Expiration**: access_token < refresh_token
4. **Password Length**: min_length >= 8
5. **Rate Limits**: All values > 0
6. **File Paths**: JWT key files must exist and be readable

### Validation Errors

Configuration validation errors prevent application startup:

```
Error: Invalid configuration
  - server.port must be between 1024 and 65535 (got: 80)
  - security.password_min_length must be at least 8 (got: 6)
  - security.jwt_private_key_path: file not found
```

---

## Configuration Best Practices

### Security

1. **Never commit secrets** to version control
2. **Use environment variables** for sensitive data
3. **Rotate secrets regularly** (90 days)
4. **Use strong JWT keys** (2048-bit RSA minimum)
5. **Enable audit logging** in production

### Performance

1. **Tune connection pools** based on load
2. **Enable Redis caching** for production
3. **Adjust worker threads** to CPU cores
4. **Set appropriate timeouts** to prevent resource exhaustion

### Observability

1. **Enable structured logging** (JSON format)
2. **Configure log levels** appropriately (info in production)
3. **Enable telemetry** for monitoring
4. **Set up distributed tracing** for debugging

---

## Environment-Specific Examples

### Local Development

```bash
# .env file
AUTH_ENVIRONMENT=development
DATABASE_URL=sqlite:./dev.db
RUST_LOG=debug
AUTH__SERVER__PORT=3000
AUTH__LOGGING__FORMAT=pretty
```

### Docker Compose

```yaml
version: '3.8'
services:
  auth-platform:
    environment:
      - AUTH_ENVIRONMENT=production
      - DATABASE_URL=mysql://auth:password@mysql:3306/auth_platform
      - AUTH__SECURITY__JWT_PRIVATE_KEY_PATH=/run/secrets/jwt_private_key
      - AUTH__CACHE__REDIS_URL=redis://redis:6379/0
    secrets:
      - jwt_private_key
      - jwt_public_key
```

### Kubernetes

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: auth-config
data:
  AUTH_ENVIRONMENT: "production"
  AUTH__SERVER__HOST: "0.0.0.0"
  AUTH__SERVER__PORT: "8080"
  AUTH__LOGGING__FORMAT: "json"
---
apiVersion: v1
kind: Secret
metadata:
  name: auth-secrets
type: Opaque
data:
  DATABASE_URL: <base64-encoded>
  JWT_PRIVATE_KEY: <base64-encoded>
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Configuration Sources**: 4 (env vars, local, environment, default)  
**Environment Variables**: 40+ configurable options
