# Directory & Module Structure

This document provides a comprehensive map of the Enterprise SSO Platform's directory structure, explaining the purpose of each folder and file organization.

## Repository Root Structure

```
sso/
├── .env                          # Environment variables (gitignored)
├── .env.example                  # Environment variable template
├── .gitignore                    # Git ignore rules
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock                    # Dependency lock file
├── Dockerfile                    # Container image definition
├── docker-compose.yml            # Local development stack
├── Makefile                      # Build and deployment commands
├── README.md                     # Project overview and quick start
├── PROJECT_STATUS.md             # Current development status
│
├── config/                       # Configuration files
│   ├── default.toml              # Base configuration
│   ├── development.toml          # Development overrides
│   ├── production.toml           # Production settings
│   └── local.toml                # Local overrides (gitignored)
│
├── crates/                       # Workspace crates
│   ├── auth-api/                 # HTTP API layer
│   ├── auth-audit/               # Audit logging
│   ├── auth-cache/               # Caching layer
│   ├── auth-config/              # Configuration management
│   ├── auth-core/                # Business logic
│   ├── auth-crypto/              # Cryptographic operations
│   ├── auth-db/                  # Database layer
│   ├── auth-extension/           # Extension framework
│   ├── auth-protocols/           # Protocol implementations
│   └── auth-telemetry/           # Observability
│
├── docs/                         # Documentation
│   ├── README.md                 # Documentation index
│   ├── code/                     # Technical documentation
│   ├── 01_product/               # Product specifications
│   ├── 03_engineering/           # Engineering docs
│   ├── 05_multitenancy/          # Multi-tenancy design
│   ├── 06_api_contracts/         # API documentation
│   ├── 07_operations/            # Operations guides
│   └── 08_governance/            # Governance policies
│
├── k8s/                          # Kubernetes manifests
│   ├── deployment.yaml           # Application deployment
│   ├── service.yaml              # Service definition
│   ├── ingress.yaml              # Ingress configuration
│   └── configmap.yaml            # Configuration map
│
├── migrations/                   # Database migrations
│   ├── 20240101000000_initial_schema.sql
│   ├── 20240102000000_add_mfa.sql
│   └── ...
│
├── scripts/                      # Utility scripts
│   ├── setup.sh                  # Development setup
│   ├── migrate.sh                # Run migrations
│   ├── test.sh                   # Run tests
│   └── deploy.sh                 # Deployment script
│
├── src/                          # Main application source
│   ├── main.rs                   # Application entry point
│   └── bin/                      # Test binaries
│       ├── test_auth_flow.rs
│       ├── test_mysql.rs
│       └── ...
│
└── target/                       # Build artifacts (gitignored)
```

---

## Crate Structure Details

### auth-core (Business Logic)

```
crates/auth-core/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Crate entry point
    ├── error.rs                  # Error types
    ├── models.rs                 # Model re-exports
    │
    ├── models/                   # Domain models
    │   ├── mod.rs                # Module exports
    │   ├── user.rs               # User entity
    │   ├── token.rs              # Token structures
    │   ├── session.rs            # Session entity
    │   ├── role.rs               # RBAC roles
    │   ├── permission.rs         # Permissions
    │   ├── tenant.rs             # Tenant model
    │   ├── organization.rs       # Organization hierarchy
    │   ├── password_policy.rs    # Password rules
    │   ├── subscription.rs       # Subscription tiers
    │   └── user_tenant.rs        # User-tenant mapping
    │
    └── services/                 # Application services
        ├── mod.rs                # Service exports
        ├── identity.rs           # Identity management
        ├── authorization.rs      # Authorization logic
        ├── credential.rs         # Credential management
        ├── token_service.rs      # Token operations
        ├── session_service.rs    # Session management
        ├── role_service.rs       # Role management
        ├── risk_assessment.rs    # Risk evaluation
        └── subscription_service.rs # Subscription management
```

**Architectural Layer**: Domain + Application  
**Dependencies**: Minimal (auth-config, auth-crypto)  
**Responsibility**: Pure business logic, no I/O

---

### auth-api (HTTP Layer)

```
crates/auth-api/
├── Cargo.toml
└── src/
    ├── lib.rs                    # API entry point, OpenAPI docs
    ├── error.rs                  # HTTP error responses
    ├── router.rs                 # Route configuration
    ├── routes.rs                 # Route definitions
    ├── server.rs                 # Server initialization
    ├── validation.rs             # Request validation
    │
    ├── handlers/                 # HTTP handlers
    │   ├── mod.rs                # Handler exports
    │   ├── auth.rs               # Authentication endpoints
    │   ├── auth_oidc.rs          # OIDC endpoints
    │   ├── auth_saml.rs          # SAML endpoints
    │   ├── health.rs             # Health checks
    │   └── users.rs              # User management
    │
    └── middleware/               # HTTP middleware
        ├── mod.rs                # Middleware exports
        ├── rate_limit.rs         # Rate limiting
        ├── request_id.rs         # Request ID generation
        └── security_headers.rs   # Security headers
```

**Architectural Layer**: Adapter (HTTP)  
**Dependencies**: auth-core, auth-db, auth-protocols, auth-telemetry  
**Responsibility**: HTTP request/response handling

---

### auth-db (Database Layer)

```
crates/auth-db/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Database entry point
    ├── connection.rs             # Connection pooling
    │
    └── repositories/             # Repository pattern
        ├── mod.rs                # Repository exports
        ├── user_repository.rs    # User CRUD
        ├── token_repository.rs   # Token storage
        ├── session_repository.rs # Session persistence
        ├── role_repository.rs    # Role storage
        ├── tenant_repository.rs  # Tenant management
        ├── organization_repository.rs # Organization CRUD
        └── subscription_repository.rs # Subscription storage
```

**Architectural Layer**: Adapter (Persistence)  
**Dependencies**: auth-core, auth-config  
**Responsibility**: Database operations, repository implementations

---

### auth-protocols (Protocol Implementations)

```
crates/auth-protocols/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Protocol entry point
    ├── oauth.rs                  # OAuth 2.1 implementation
    ├── oidc.rs                   # OpenID Connect
    ├── saml.rs                   # SAML 2.0
    └── scim.rs                   # SCIM 2.0
```

**Architectural Layer**: Adapter (Protocol)  
**Dependencies**: auth-core, auth-config, auth-crypto  
**Responsibility**: Authentication protocol implementations

---

### auth-config (Configuration)

```
crates/auth-config/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Config entry point
    ├── config.rs                 # Configuration structures
    ├── loader.rs                 # Multi-source loading
    ├── manager.rs                # Runtime management
    └── validation.rs             # Configuration validation
```

**Architectural Layer**: Infrastructure  
**Dependencies**: None (foundation)  
**Responsibility**: Configuration management

---

### auth-crypto (Cryptography)

```
crates/auth-crypto/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Crypto entry point
    ├── password.rs               # Password hashing (Argon2id)
    ├── jwt.rs                    # JWT operations
    ├── keys.rs                   # Key management
    ├── encryption.rs             # Encryption/decryption
    ├── signatures.rs             # Digital signatures
    └── pqc.rs                    # Post-quantum crypto
```

**Architectural Layer**: Infrastructure  
**Dependencies**: None (foundation)  
**Responsibility**: Cryptographic operations

---

### auth-cache (Caching)

```
crates/auth-cache/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Cache trait definitions
    └── memory.rs                 # In-memory cache (DashMap)
```

**Architectural Layer**: Infrastructure  
**Dependencies**: None  
**Responsibility**: Multi-layer caching

---

### auth-telemetry (Observability)

```
crates/auth-telemetry/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Telemetry entry point
    ├── metrics.rs                # Prometheus metrics
    ├── tracing.rs                # Distributed tracing
    ├── health.rs                 # Health monitoring
    └── performance.rs            # Performance metrics
```

**Architectural Layer**: Infrastructure  
**Dependencies**: None  
**Responsibility**: Metrics, tracing, monitoring

---

### auth-audit (Audit Logging)

```
crates/auth-audit/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Audit entry point
    ├── events.rs                 # Event definitions
    ├── logger.rs                 # Audit logger
    ├── storage.rs                # Persistent storage
    ├── service.rs                # Audit service
    └── compliance.rs             # Compliance reporting
```

**Architectural Layer**: Infrastructure  
**Dependencies**: auth-core, auth-config, auth-crypto  
**Responsibility**: Audit logging, compliance

---

### auth-extension (Extensions)

```
crates/auth-extension/
├── Cargo.toml
└── src/
    ├── lib.rs                    # Extension entry point
    ├── graphql.rs                # GraphQL API
    ├── scripting.rs              # Rhai scripting
    ├── webhooks.rs               # Webhook system
    └── plugins.rs                # Plugin architecture
```

**Architectural Layer**: Adapter (Extension)  
**Dependencies**: auth-core  
**Responsibility**: GraphQL API, scripting, extensibility

---

## Architectural Layers

The codebase follows a layered architecture with strict dependency rules:

### Layer 1: Domain (Pure Business Logic)
**Location**: `auth-core/src/models/`  
**Dependencies**: None  
**Contains**: Entities, value objects, domain events

### Layer 2: Application (Use Cases)
**Location**: `auth-core/src/services/`  
**Dependencies**: Domain layer only  
**Contains**: Business logic orchestration, service interfaces

### Layer 3: Adapters (External Interfaces)
**Locations**:
- `auth-api/` - HTTP adapter
- `auth-db/` - Database adapter
- `auth-protocols/` - Protocol adapter
- `auth-extension/` - Extension adapter

**Dependencies**: Application + Domain layers  
**Contains**: Interface implementations, external integrations

### Layer 4: Infrastructure (Cross-Cutting Concerns)
**Locations**:
- `auth-config/` - Configuration
- `auth-crypto/` - Cryptography
- `auth-cache/` - Caching
- `auth-telemetry/` - Observability
- `auth-audit/` - Audit logging

**Dependencies**: Minimal, supports all layers  
**Contains**: Technical capabilities, utilities

---

## Dependency Direction

```
┌─────────────────────────────────────────┐
│         Infrastructure Layer            │
│  (config, crypto, cache, telemetry)     │
└─────────────────────────────────────────┘
                  ↑
┌─────────────────────────────────────────┐
│           Adapter Layer                 │
│  (api, db, protocols, extension)        │
└─────────────────────────────────────────┘
                  ↑
┌─────────────────────────────────────────┐
│        Application Layer                │
│      (auth-core/services)               │
└─────────────────────────────────────────┘
                  ↑
┌─────────────────────────────────────────┐
│          Domain Layer                   │
│       (auth-core/models)                │
└─────────────────────────────────────────┘
```

**Rules**:
1. Domain layer has zero dependencies on other layers
2. Application layer depends only on Domain
3. Adapters depend on Application + Domain
4. Infrastructure supports all layers without creating circular dependencies
5. No horizontal dependencies between adapters

---

## File Naming Conventions

### Rust Files
- **Modules**: `snake_case.rs` (e.g., `user_repository.rs`)
- **Entry Points**: `lib.rs` for libraries, `main.rs` for binaries
- **Module Declarations**: `mod.rs` for folder modules
- **Tests**: `#[cfg(test)]` modules in same file or `tests/` directory

### Configuration Files
- **TOML**: `lowercase.toml` (e.g., `default.toml`)
- **Environment**: `.env` for local, `.env.example` for template

### Documentation
- **Markdown**: `UPPERCASE.md` for root-level (e.g., `README.md`)
- **Markdown**: `lowercase.md` for nested docs (e.g., `crates.md`)

### Database Migrations
- **Format**: `YYYYMMDDHHMMSS_description.sql`
- **Example**: `20240101120000_create_users_table.sql`

---

## Module Organization Patterns

### Repository Pattern (auth-db)
```rust
// repositories/user_repository.rs
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    pub fn new(pool: MySqlPool) -> Self { ... }
}

#[async_trait]
impl UserStore for UserRepository {
    async fn find_by_email(...) -> Result<...> { ... }
}
```

### Service Pattern (auth-core)
```rust
// services/identity.rs
pub struct IdentityService {
    store: Arc<dyn UserStore>,
    token_service: Arc<dyn TokenProvider>,
}

impl IdentityService {
    pub fn new(...) -> Self { ... }
    pub async fn register(...) -> Result<...> { ... }
    pub async fn login(...) -> Result<...> { ... }
}
```

### Handler Pattern (auth-api)
```rust
// handlers/auth.rs
#[utoipa::path(post, path = "/auth/login")]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Handler implementation
}
```

---

## Test Organization

### Unit Tests
**Location**: Same file as implementation  
**Pattern**: `#[cfg(test)] mod tests { ... }`

### Integration Tests
**Location**: `tests/` directory in each crate  
**Pattern**: `tests/integration_test.rs`

### Property-Based Tests
**Location**: Same file or separate `property_tests.rs`  
**Pattern**: Uses `proptest` crate

### Test Binaries
**Location**: `src/bin/test_*.rs`  
**Purpose**: End-to-end testing, manual verification

---

## Build Artifacts

### Target Directory Structure
```
target/
├── debug/                        # Debug builds
│   ├── auth-platform             # Main binary
│   ├── test_*                    # Test binaries
│   └── deps/                     # Dependencies
│
├── release/                      # Release builds
│   └── auth-platform             # Optimized binary
│
└── doc/                          # Generated documentation
    └── auth_*/                   # Crate documentation
```

---

## Configuration Directory

```
config/
├── default.toml                  # Base configuration (committed)
├── development.toml              # Dev overrides (committed)
├── production.toml               # Prod settings (committed)
└── local.toml                    # Local overrides (gitignored)
```

**Precedence**: `local.toml` > `{environment}.toml` > `default.toml`

---

## Migration Directory

```
migrations/
├── 20240101000000_initial_schema.sql
├── 20240102000000_add_mfa_support.sql
├── 20240103000000_add_tenants.sql
├── 20240104000000_add_rbac.sql
└── 20240105000000_add_audit_logs.sql
```

**Managed By**: SQLx migrations  
**Applied**: Automatically on application startup  
**Rollback**: Manual via SQL scripts

---

## Documentation Directory

```
docs/
├── README.md                     # Documentation index
├── code/                         # Technical documentation
│   ├── crates.md                 # Crate documentation
│   ├── structure.md              # This file
│   ├── flows.md                  # Execution flows
│   ├── security.md               # Security documentation
│   ├── dependencies.md           # Dependency graph
│   ├── configuration.md          # Configuration guide
│   └── crates/                   # File-by-file docs
│       ├── auth-core/
│       ├── auth-api/
│       └── ...
│
├── 01_product/                   # Product specifications
├── 03_engineering/               # Engineering documentation
├── 05_multitenancy/              # Multi-tenancy design
├── 06_api_contracts/             # API contracts
├── 07_operations/                # Operations guides
└── 08_governance/                # Governance policies
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Total Directories**: 25+  
**Total Files**: 109 Rust files + configuration + documentation
