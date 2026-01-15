# main.rs

## File Metadata

**File Path**: `src/main.rs`  
**Crate**: `auth-platform` (root)  
**Module**: Binary entry point  
**Layer**: Infrastructure  
**Security-Critical**: ✅ **YES** - Application bootstrap and initialization

## Purpose

Main application entry point that initializes all services, establishes database connections, runs migrations, and starts the HTTP server.

### Problem It Solves

- Bootstraps the entire SSO platform
- Initializes dependency injection container
- Manages application lifecycle
- Configures logging and observability
- Establishes database connections

---

## Detailed Code Breakdown

### Initialization Sequence

#### 1. Environment Variables
```rust
dotenvy::dotenv().ok();
```
**Purpose**: Load `.env` file for local development  
**Variables**: `DATABASE_URL`, `AUTH_ENVIRONMENT`, etc.

---

#### 2. Logging Setup
```rust
tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "auth_platform=debug,auth_api=debug,tower_http=debug".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

**Components**:
- **EnvFilter**: Configurable log levels via `RUST_LOG` env var
- **Default**: `debug` level for auth crates, `tower_http`
- **Format**: Structured JSON logs (production) or pretty-printed (development)

**Log Levels**:
```bash
# Development
RUST_LOG=debug

# Production
RUST_LOG=info,auth_core=debug,auth_api=info
```

---

#### 3. Configuration Loading
```rust
let environment = std::env::var("AUTH_ENVIRONMENT")
    .unwrap_or_else(|_| "development".to_string());
let config_loader = ConfigLoader::new("config", &environment);
let config_manager = ConfigManager::new(config_loader)?;
let config = config_manager.get_config();
```

**Configuration Sources** (in precedence order):
1. Environment variables (`AUTH__DATABASE__MAX_CONNECTIONS`)
2. `config/local.toml` (gitignored)
3. `config/{environment}.toml` (development.toml, production.toml)
4. `config/default.toml`

**Example Configuration**:
```toml
# config/production.toml
[database]
max_connections = 20
min_connections = 5

[server]
host = "0.0.0.0"
port = 8080

[jwt]
access_token_ttl_seconds = 900
refresh_token_ttl_days = 30
```

---

#### 4. Database Connection
```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
let pool = MySqlPoolOptions::new()
    .max_connections(config.database.max_connections)
    .connect(&database_url)
    .await
    .expect("Failed to connect to database");
```

**Connection Pool**:
- **Type**: MySQL connection pool
- **Max Connections**: From config (default: 10)
- **URL Format**: `mysql://user:password@localhost:3306/auth_db`

**Error Handling**: Panics if connection fails (fail-fast on startup)

---

#### 5. Database Migrations
```rust
sqlx::migrate!()
    .run(&pool)
    .await
    .expect("Failed to run migrations");
```

**Migration System**:
- **Location**: `migrations/` directory
- **Format**: SQL files with timestamps
- **Execution**: Automatic on startup
- **Idempotent**: Safe to run multiple times

**Example Migration**:
```sql
-- migrations/20240101000000_create_users_table.sql
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

---

#### 6. Repository Initialization
```rust
let role_repo = Arc::new(RoleRepository::new(pool.clone()));
let session_repo = Arc::new(SessionRepository::new(pool.clone()));
let subscription_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
let user_repo = Arc::new(UserRepository::new(pool.clone()));
```

**Pattern**: Repository pattern with trait abstraction

**Arc Wrapping**: Enables shared ownership across async tasks

---

#### 7. Service Initialization
```rust
let role_service = Arc::new(RoleService::new(role_repo));
let risk_engine = Arc::new(RiskEngine::new());
let session_service = Arc::new(SessionService::new(session_repo, risk_engine));
let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));
```

**Dependency Injection**: Services depend on repositories (injected via constructor)

---

#### 8. Token Service Initialization
```rust
let token_service: Arc<dyn TokenProvider> = 
    Arc::new(TokenEngine::new().await
        .expect("Failed to initialize TokenEngine"));
```

**Type**: Trait object for flexibility  
**Implementation**: `TokenEngine` with in-memory stores (development)  
**Production**: Replace with database-backed stores

---

#### 9. Identity Service Initialization
```rust
let identity_service = Arc::new(IdentityService::new(
    user_repo as Arc<dyn UserStore>,
    token_service,
));
```

**Dependencies**:
- `UserStore` trait: User persistence
- `TokenProvider` trait: Token operations

---

#### 10. Application State
```rust
let app_state = AppState {
    db: pool,
    role_service,
    session_service,
    subscription_service,
    identity_service,
};
```

**Purpose**: Shared state passed to all HTTP handlers

---

#### 11. Router Initialization
```rust
let app = auth_api::app(app_state);
```

**Returns**: Configured Axum router with:
- All API routes
- Middleware stack
- Swagger UI

---

#### 12. Server Start
```rust
let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
info!("Server listening on {}", addr);

let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
axum::serve(listener, app).await.unwrap();
```

**Bind Address**: `127.0.0.1:8081` (localhost only)  
**Production**: Bind to `0.0.0.0:8080` for external access

---

## Startup Sequence Diagram

```mermaid
sequenceDiagram
    participant Main
    participant Config
    participant Database
    participant Repos
    participant Services
    participant Server
    
    Main->>Main: Load .env
    Main->>Main: Initialize logging
    Main->>Config: Load configuration
    Config-->>Main: Config object
    Main->>Database: Connect to MySQL
    Database-->>Main: Connection pool
    Main->>Database: Run migrations
    Main->>Repos: Initialize repositories
    Repos-->>Main: Repository instances
    Main->>Services: Initialize services
    Services-->>Main: Service instances
    Main->>Server: Start HTTP server
    Server-->>Main: Listening on :8081
```

---

## Error Handling

### Fail-Fast Strategy

**Philosophy**: Fail immediately on startup if critical components unavailable

**Examples**:
```rust
// Database connection
.expect("Failed to connect to database")

// Migrations
.expect("Failed to run migrations")

// Token engine
.expect("Failed to initialize TokenEngine")
```

**Rationale**:
- Prevents running with misconfigured state
- Clear error messages for operators
- Faster debugging

---

## Production Considerations

### 1. Graceful Shutdown

**Current**: No graceful shutdown  
**Needed**:
```rust
let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

// Handle SIGTERM
tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    shutdown_tx.send(()).ok();
});

// Serve with shutdown signal
axum::serve(listener, app)
    .with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    })
    .await
    .unwrap();
```

### 2. Health Checks

**Endpoint**: `GET /health`  
**Checks**:
- Database connectivity
- Service availability

### 3. Metrics

**Integration**: Prometheus metrics  
**Endpoint**: `GET /metrics`

### 4. Configuration Validation

**Add**:
```rust
config.validate()?;
```

**Checks**:
- Required fields present
- Valid ranges (e.g., max_connections > 0)
- URL formats

---

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | MySQL connection string | `mysql://user:pass@localhost:3306/auth_db` |

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `AUTH_ENVIRONMENT` | Environment name | `development` |
| `RUST_LOG` | Log level | `info` |
| `SERVER_PORT` | HTTP port | `8080` |

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `anyhow` | Error handling |
| `tracing` | Structured logging |
| `tracing-subscriber` | Log formatting |
| `sqlx` | Database connection |
| `dotenvy` | .env file loading |
| `axum` | HTTP server |

### Internal Crates

- [auth-config](file:///c:/Users/Victo/Downloads/sso/crates/auth-config) - Configuration management
- [auth-core](file:///c:/Users/Victo/Downloads/sso/crates/auth-core) - Business logic
- [auth-db](file:///c:/Users/Victo/Downloads/sso/crates/auth-db) - Database layer
- [auth-api](file:///c:/Users/Victo/Downloads/sso/crates/auth-api) - HTTP API

---

## Testing

### Integration Tests

**Location**: `tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_server_starts() {
    // Start server in background
    let handle = tokio::spawn(async {
        main().await.unwrap();
    });
    
    // Wait for server to be ready
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Test health endpoint
    let response = reqwest::get("http://localhost:8081/health").await.unwrap();
    assert_eq!(response.status(), 200);
    
    handle.abort();
}
```

---

## Related Files

- [auth-api/lib.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - HTTP API setup
- [auth-config](file:///c:/Users/Victo/Downloads/sso/crates/auth-config) - Configuration loading
- [Cargo.toml](file:///c:/Users/Victo/Downloads/sso/Cargo.toml) - Workspace configuration

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 112  
**Security Level**: CRITICAL
