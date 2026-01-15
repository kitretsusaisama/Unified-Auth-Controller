---
title: Engineering Constitution v1.0
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Engineering Standards
---

# Engineering Constitution v1.0

> [!IMPORTANT]
> **This is the single source of truth for all engineering decisions in UPFlame Unified Auth Controller.**
> 
> Every code change, architecture decision, and AI prompt MUST reference and comply with this document.

---

## 1. Core Principles

### 1.1 Non-Negotiable Values

1. **Security First**: Security is not a feature - it's a requirement
2. **Zero Trust**: Never trust, always verify
3. **Fail Secure**: Systems fail closed, not open
4. **Explicit Over Implicit**: No magic, no surprises
5. **Testability**: If it can't be tested, it doesn't exist
6. **Observability**: Every action must be traceable

### 1.2 Engineering Philosophy

- **Boring Technology**: Prefer proven, stable technologies over bleeding edge
- **Composition Over Inheritance**: Favor traits and composition
- **Immutability by Default**: Mutable state is explicit and justified
- **Error Handling is Not Optional**: Every error path is handled
- **Documentation is Code**: Outdated docs are worse than no docs

---

## 2. Rust Coding Standards

### 2.1 Style Guide

#### Formatting

```bash
# MANDATORY: Run before every commit
cargo fmt --all

# MANDATORY: Fix all clippy warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**Rules**:
- **Line Length**: 100 characters maximum
- **Indentation**: 4 spaces (enforced by rustfmt)
- **Imports**: Group by `std`, `external crates`, `internal crates`, `self`
- **Naming**:
  - Types: `PascalCase` (e.g., `UserStore`, `AuthError`)
  - Functions/Methods: `snake_case` (e.g., `find_by_email`, `issue_token`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_LOGIN_ATTEMPTS`)
  - Lifetimes: Single letter `'a`, `'b` or descriptive `'conn`

#### Code Organization

```rust
// ✅ CORRECT: Imports grouped and sorted
use std::sync::Arc;
use std::collections::HashMap;

use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AuthError;
use crate::models::User;

// ❌ WRONG: Unsorted, mixed groups
use crate::error::AuthError;
use std::sync::Arc;
use uuid::Uuid;
```

### 2.2 Error Handling

#### Error Types

**MANDATORY**: Use `thiserror` for domain errors, `anyhow` for application errors.

```rust
// ✅ CORRECT: Domain error with thiserror
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Account locked until {0}")]
    AccountLocked(DateTime<Utc>),
    
    #[error("Validation failed: {message}")]
    ValidationError { message: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

// ✅ CORRECT: Application error with anyhow
use anyhow::{Context, Result};

pub async fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    Ok(())
}
```

#### Error Handling Rules

1. **Never use `.unwrap()` or `.expect()` in production code**
   - Exception: Test code and initialization code with clear justification
2. **Always propagate errors with context**
   - Use `.context()` or `.with_context()` from anyhow
3. **Log errors at the point of handling, not creation**
4. **Include request IDs in all error responses**

```rust
// ✅ CORRECT: Proper error handling
pub async fn find_user(&self, id: Uuid) -> Result<User, AuthError> {
    self.db.query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AuthError::UserNotFound { id })
}

// ❌ WRONG: Using unwrap
pub async fn find_user(&self, id: Uuid) -> User {
    self.db.query_as("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap() // NEVER DO THIS
}
```

### 2.3 Async Patterns

#### Async Trait Usage

```rust
use async_trait::async_trait;

// ✅ CORRECT: Async trait for repository
#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError>;
    async fn create(&self, user: CreateUserRequest) -> Result<User, AuthError>;
}

// ✅ CORRECT: Implementation
#[async_trait]
impl UserStore for UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
        // Implementation
    }
}
```

#### Concurrency Rules

1. **Use `Arc` for shared ownership across async boundaries**
2. **Use `tokio::sync::RwLock` or `parking_lot::RwLock` for shared mutable state**
3. **Prefer message passing (channels) over shared state**
4. **Always set timeouts for external calls**

```rust
// ✅ CORRECT: Shared service with Arc
pub struct IdentityService {
    store: Arc<dyn UserStore>,
    token_service: Arc<dyn TokenProvider>,
}

// ✅ CORRECT: Timeout for external call
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    external_api_call()
).await??;
```

### 2.4 Type Safety

#### Newtype Pattern

**MANDATORY**: Use newtype pattern for domain primitives.

```rust
// ✅ CORRECT: Newtype for email
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(email: impl Into<String>) -> Result<Self, ValidationError> {
        let email = email.into();
        // Validation logic
        Ok(Self(email))
    }
}

// ❌ WRONG: Raw string for email
pub struct User {
    pub email: String, // No validation, easy to misuse
}
```

#### Builder Pattern

Use for complex construction:

```rust
// ✅ CORRECT: Builder for complex types
#[derive(Default)]
pub struct UserBuilder {
    email: Option<Email>,
    password: Option<String>,
    tenant_id: Option<Uuid>,
}

impl UserBuilder {
    pub fn email(mut self, email: Email) -> Self {
        self.email = Some(email);
        self
    }
    
    pub fn build(self) -> Result<CreateUserRequest, ValidationError> {
        Ok(CreateUserRequest {
            email: self.email.ok_or(ValidationError::MissingEmail)?,
            password: self.password,
            tenant_id: self.tenant_id.ok_or(ValidationError::MissingTenantId)?,
        })
    }
}
```

---

## 3. Project Structure

### 3.1 Workspace Organization

```
auth-platform/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── auth-core/          # Pure business logic (NO I/O)
│   ├── auth-api/           # HTTP layer (Axum)
│   ├── auth-protocols/     # OIDC, SAML, OAuth
│   ├── auth-db/            # Database layer
│   ├── auth-config/        # Configuration management
│   ├── auth-crypto/        # Cryptographic operations
│   ├── auth-cache/         # Caching layer
│   ├── auth-telemetry/     # Observability
│   ├── auth-audit/         # Audit logging
│   └── auth-extension/     # GraphQL, scripting
├── config/                 # Configuration files
├── migrations/             # Database migrations
├── scripts/                # Build/deployment scripts
├── docs/                   # Documentation
└── src/
    ├── main.rs             # Application entry point
    └── bin/                # Test binaries
```

### 3.2 Crate Structure

**MANDATORY**: Every crate follows this structure:

```
crates/auth-core/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API exports
│   ├── error.rs            # Error types
│   ├── models/             # Data models
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── tenant.rs
│   ├── services/           # Business logic
│   │   ├── mod.rs
│   │   ├── identity.rs
│   │   └── authorization.rs
│   └── utils/              # Utilities
└── tests/                  # Integration tests
    └── integration_test.rs
```

### 3.3 Module Organization Rules

1. **`lib.rs` is the public API** - only export what's needed
2. **`mod.rs` re-exports submodules** - flat public API
3. **Private by default** - use `pub` sparingly
4. **One concept per file** - `user.rs` contains only User-related code

```rust
// ✅ CORRECT: lib.rs exports clean API
pub mod error;
pub mod models;
pub mod services;

pub use error::AuthError;
pub use models::{User, Tenant, Role};
pub use services::{IdentityService, AuthorizationEngine};

// Internal modules not exported
mod utils;
```

---

## 4. Security Rules

### 4.1 Secrets Management

**CRITICAL RULES**:

1. ❌ **NEVER commit secrets to Git**
2. ✅ **All secrets via environment variables**
3. ✅ **Use `.env.example` with placeholder values**
4. ✅ **Secrets wrapped in `secrecy::Secret<T>`**

```rust
use secrecy::{Secret, ExposeSecret};

// ✅ CORRECT: Secret wrapper
pub struct Config {
    pub jwt_secret: Secret<String>,
    pub database_url: Secret<String>,
}

// ✅ CORRECT: Only expose when needed
fn sign_token(&self, claims: Claims) -> Result<String> {
    let secret = self.config.jwt_secret.expose_secret();
    // Use secret
}
```

### 4.2 Password Security

**MANDATORY RULES**:

1. **Always use Argon2id** for password hashing
2. **Never log passwords** (even hashed)
3. **Minimum 12 characters** enforced
4. **Complexity requirements** enforced

```rust
use argon2::{Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use argon2::password_hash::{SaltString, rand_core::OsRng};

// ✅ CORRECT: Argon2id hashing
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::CryptoError(e.to_string()))?
        .to_string();
    
    Ok(hash)
}

// ✅ CORRECT: Constant-time verification
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::CryptoError(e.to_string()))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

### 4.3 JWT Token Security

**MANDATORY RULES**:

1. **Use RS256** (asymmetric) for access tokens
2. **Short expiration** - 15 minutes for access tokens
3. **Refresh tokens** - 30 days, stored in database
4. **Token revocation** - maintain revoked token list

```rust
// ✅ CORRECT: Token configuration
pub struct TokenConfig {
    pub access_token_ttl: Duration,      // 15 minutes
    pub refresh_token_ttl: Duration,     // 30 days
    pub algorithm: Algorithm,            // RS256
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            access_token_ttl: Duration::minutes(15),
            refresh_token_ttl: Duration::days(30),
            algorithm: Algorithm::RS256,
        }
    }
}
```

### 4.4 SQL Injection Prevention

**MANDATORY**: Use `sqlx` compile-time checked queries.

```rust
// ✅ CORRECT: Parameterized query with sqlx
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE email = ? AND tenant_id = ?"
)
.bind(&email)
.bind(&tenant_id)
.fetch_optional(&pool)
.await?;

// ❌ WRONG: String interpolation (SQL injection risk)
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

### 4.5 Input Validation

**MANDATORY**: Validate all inputs at API boundary.

```rust
use validator::{Validate, ValidationError};

// ✅ CORRECT: Validation with validator crate
#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 12, max = 128))]
    pub password: Option<String>,
    
    #[validate(phone)]
    pub phone: Option<String>,
}

// ✅ CORRECT: Validate before processing
pub async fn register(
    Json(request): Json<CreateUserRequest>
) -> Result<Json<User>, ApiError> {
    request.validate()?;  // Validate first
    // Process...
}
```

---

## 5. Testing Requirements

### 5.1 Test Coverage

**MANDATORY MINIMUMS**:

- **Unit Tests**: 80% coverage for business logic
- **Integration Tests**: All API endpoints
- **Property Tests**: All cryptographic operations
- **Load Tests**: Before production deployment

### 5.2 Test Organization

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_password_hashing() {
        // Test implementation
    }
}

// Integration tests in tests/ directory
// tests/identity_service_test.rs
#[tokio::test]
async fn test_user_registration() {
    // Integration test
}
```

### 5.3 Property-Based Testing

**MANDATORY**: Use `proptest` for cryptographic and security-critical code.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_password_hash_uniqueness(password in "\\PC{12,128}") {
        let hash1 = hash_password(&password)?;
        let hash2 = hash_password(&password)?;
        
        // Same password should produce different hashes (due to salt)
        prop_assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        prop_assert!(verify_password(&password, &hash1)?);
        prop_assert!(verify_password(&password, &hash2)?);
    }
}
```

### 5.4 Test Data

**RULES**:

1. **Never use production data** in tests
2. **Use factories** for test data generation
3. **Clean up after tests** (database, files)

```rust
// ✅ CORRECT: Test data factory
pub fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: format!("test-{}@example.com", Uuid::new_v4()),
        tenant_id: Uuid::new_v4(),
        status: UserStatus::Active,
        created_at: Utc::now(),
        ..Default::default()
    }
}
```

---

## 6. Dependency Management

### 6.1 Workspace Dependencies

**MANDATORY**: All dependencies defined in workspace `Cargo.toml`.

```toml
# ✅ CORRECT: Workspace-level dependencies
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "mysql"] }
```

```toml
# ✅ CORRECT: Crate uses workspace dependency
[dependencies]
tokio = { workspace = true }
sqlx = { workspace = true }
```

### 6.2 Dependency Audit

**MANDATORY**: Run security audit before every release.

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Fail build on vulnerabilities
cargo audit --deny warnings
```

### 6.3 Update Policy

1. **Patch updates**: Apply immediately (automated)
2. **Minor updates**: Monthly review and update
3. **Major updates**: Quarterly review, requires testing
4. **Security updates**: Apply within 24 hours

---

## 7. Code Review Rules

### 7.1 Pull Request Requirements

**MANDATORY CHECKLIST**:

- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated
- [ ] Changelog entry added
- [ ] Security implications reviewed

### 7.2 Review Levels

| Change Type | Reviewers Required | Additional Requirements |
|-------------|-------------------|------------------------|
| Bug fix | 1 | Tests added |
| Feature | 1 | Documentation updated |
| Security | 2 | Security team approval |
| Breaking change | 2 | ADR created |
| Database migration | 2 | Rollback plan |

### 7.3 Review Guidelines

**Reviewers MUST check**:

1. **Security**: No secrets, proper input validation
2. **Error Handling**: All errors handled
3. **Testing**: Adequate test coverage
4. **Performance**: No obvious performance issues
5. **Maintainability**: Code is readable and documented

---

## 8. Documentation Standards

### 8.1 Code Documentation

**MANDATORY**: All public APIs have doc comments.

```rust
/// Authenticates a user with email and password.
///
/// # Arguments
///
/// * `request` - Authentication request containing email and password
///
/// # Returns
///
/// Returns `AuthResponse` with user details and tokens on success.
///
/// # Errors
///
/// * `AuthError::InvalidCredentials` - Email or password is incorrect
/// * `AuthError::AccountLocked` - Account is locked due to failed attempts
///
/// # Example
///
/// ```rust
/// let request = AuthRequest {
///     email: "user@example.com".to_string(),
///     password: "SecurePass123!".to_string(),
///     tenant_id: Uuid::new_v4(),
/// };
///
/// let response = identity_service.login(request).await?;
/// ```
pub async fn login(&self, request: AuthRequest) -> Result<AuthResponse, AuthError> {
    // Implementation
}
```

### 8.2 Architecture Decision Records (ADRs)

**MANDATORY**: Create ADR for significant decisions.

**When to create ADR**:
- Technology choice (e.g., "Why Rust?")
- Architecture pattern (e.g., "Modular monolith vs. microservices")
- Security decision (e.g., "Argon2id for password hashing")
- Breaking changes

**ADR Template**: See `docs/08_governance/adr_template.mdx`

---

## 9. Performance Standards

### 9.1 Response Time Targets

| Operation | Target | Maximum |
|-----------|--------|---------|
| Authentication | 50ms | 200ms |
| Token validation | 10ms | 50ms |
| Database query | 20ms | 100ms |
| API endpoint | 100ms | 500ms |

### 9.2 Optimization Rules

1. **Measure first** - No premature optimization
2. **Use connection pooling** - Database and Redis
3. **Cache aggressively** - But invalidate correctly
4. **Async all I/O** - Never block the runtime
5. **Batch operations** - Reduce round trips

```rust
// ✅ CORRECT: Connection pool configuration
let pool = sqlx::mysql::MySqlPoolOptions::new()
    .max_connections(50)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(&database_url)
    .await?;
```

---

## 10. Observability Requirements

### 10.1 Logging

**MANDATORY**: Use structured logging with `tracing`.

```rust
use tracing::{info, warn, error, instrument};

// ✅ CORRECT: Structured logging
#[instrument(skip(self, password), fields(user_id = %user.id))]
pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse> {
    info!("Login attempt for email: {}", email);
    
    match self.authenticate(email, password).await {
        Ok(response) => {
            info!("Login successful");
            Ok(response)
        }
        Err(e) => {
            warn!("Login failed: {}", e);
            Err(e)
        }
    }
}
```

**Logging Levels**:
- `error`: System errors requiring immediate attention
- `warn`: Recoverable errors, security events
- `info`: Important business events (login, registration)
- `debug`: Detailed diagnostic information
- `trace`: Very verbose debugging

### 10.2 Metrics

**MANDATORY**: Expose Prometheus metrics.

```rust
use metrics::{counter, histogram};

// ✅ CORRECT: Metrics instrumentation
pub async fn login(&self, request: AuthRequest) -> Result<AuthResponse> {
    let start = Instant::now();
    
    let result = self.authenticate(request).await;
    
    histogram!("auth.login.duration", start.elapsed());
    
    match result {
        Ok(response) => {
            counter!("auth.login.success", 1);
            Ok(response)
        }
        Err(e) => {
            counter!("auth.login.failure", 1);
            Err(e)
        }
    }
}
```

### 10.3 Distributed Tracing

**MANDATORY**: Include request IDs in all operations.

```rust
use uuid::Uuid;

// ✅ CORRECT: Request ID propagation
pub struct RequestContext {
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub tenant_id: Option<Uuid>,
}
```

---

## 11. Compliance & Audit

### 11.1 Audit Logging

**MANDATORY**: Log all security-critical events.

**Events requiring audit logs**:
- User registration
- Login (success and failure)
- Password changes
- Permission changes
- Token issuance and revocation
- Configuration changes
- Data access (for sensitive data)

```rust
// ✅ CORRECT: Audit log entry
pub async fn log_event(&self, event: AuditEvent) -> Result<()> {
    let entry = AuditLogEntry {
        id: Uuid::new_v4(),
        event_type: event.event_type,
        user_id: event.user_id,
        tenant_id: event.tenant_id,
        timestamp: Utc::now(),
        ip_address: event.ip_address,
        user_agent: event.user_agent,
        details: serde_json::to_value(event.details)?,
    };
    
    self.repository.insert(entry).await
}
```

### 11.2 PII Handling

**MANDATORY RULES**:

1. **Never log PII** (passwords, tokens, SSNs)
2. **Mask PII in logs** (email: `u***@example.com`)
3. **Encrypt PII at rest**
4. **Audit PII access**

```rust
// ✅ CORRECT: PII masking
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() > 2 {
            format!("{}***@{}", &local[..1], &domain[1..])
        } else {
            format!("***@{}", &domain[1..])
        }
    } else {
        "***".to_string()
    }
}
```

---

## 12. Deployment Standards

### 12.1 Environment Configuration

**MANDATORY**: Separate configuration per environment.

```
config/
├── default.toml       # Base configuration
├── development.toml   # Dev overrides
├── staging.toml       # Staging overrides
└── production.toml    # Production overrides
```

### 12.2 Database Migrations

**MANDATORY RULES**:

1. **Migrations are forward-only** (no down migrations in production)
2. **Migrations are idempotent**
3. **Test migrations on staging first**
4. **Backup before migration**

```sql
-- ✅ CORRECT: Idempotent migration
CREATE TABLE IF NOT EXISTS users (
    id BINARY(16) PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
```

### 12.3 Feature Flags

**RECOMMENDED**: Use feature flags for gradual rollout.

```rust
pub struct FeatureFlags {
    pub webauthn_enabled: bool,
    pub graphql_api_enabled: bool,
    pub risk_based_auth_enabled: bool,
}
```

---

## 13. AI-Assisted Development

### 13.1 AI Prompt Requirements

**MANDATORY**: Every AI prompt must reference this constitution.

**Example prompt template**:

```
Task: Implement user registration endpoint

Context:
- Follow Engineering Constitution v1.0
- Use auth-api crate for HTTP layer
- Use auth-core IdentityService for business logic
- Validate input per Section 4.5
- Return structured errors per Section 2.2

Requirements:
1. POST /auth/register endpoint
2. Validate email and password
3. Hash password with Argon2id
4. Return 201 on success, 400 on validation error
5. Include tests
```

### 13.2 Code Generation Rules

**AI-generated code MUST**:

1. Follow all coding standards (Section 2)
2. Include error handling (Section 2.2)
3. Include tests (Section 5)
4. Include documentation (Section 8)
5. Pass `cargo fmt` and `cargo clippy`

---

## 14. Enforcement

### 14.1 Automated Checks

**CI/CD Pipeline MUST include**:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all

# Security audit
cargo audit --deny warnings

# Build
cargo build --release
```

### 14.2 Manual Review

**Required for**:
- All pull requests
- Security changes (2 reviewers)
- Breaking changes (2 reviewers + ADR)

### 14.3 Violations

**Consequences**:

1. **First violation**: PR rejected, education provided
2. **Repeated violations**: Escalation to team lead
3. **Security violations**: Immediate escalation, incident review

---

## 15. Version History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0.0 | 2026-01-12 | Initial constitution | Engineering Team |

---

## 16. Acknowledgments

This constitution is inspired by:
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Google Engineering Practices](https://google.github.io/eng-practices/)
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)

---

## Appendix A: Quick Reference

### Essential Commands

```bash
# Development
cargo build
cargo run
cargo test

# Quality
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo audit

# Database
sqlx migrate run
sqlx migrate revert

# Documentation
cargo doc --open
```

### Common Patterns

```rust
// Error handling
.context("Failed to load config")?

// Async trait
#[async_trait]
pub trait Repository: Send + Sync { }

// Logging
#[instrument(skip(password))]

// Validation
request.validate()?
```

---

**END OF ENGINEERING CONSTITUTION v1.0**
