# Enterprise SSO Platform - Technical Documentation

## Project Overview

The Enterprise SSO Platform is a production-ready, security-critical authentication and identity management system built entirely in Rust. It provides comprehensive Single Sign-On (SSO) capabilities, multi-protocol support, and enterprise-grade features designed to handle 100,000 to 1,000,000+ concurrent users with sub-50ms response times.

### Project Purpose

This platform serves as a centralized authentication and authorization hub for enterprise environments, enabling:

- **Unified Identity Management**: Single source of truth for user identities across multiple applications and services
- **Multi-Protocol Authentication**: Support for SAML 2.0, OpenID Connect 1.0, OAuth 2.1, and SCIM 2.0
- **Advanced Multi-Tenancy**: Hierarchical organization → tenant → user structure for complex enterprise deployments
- **Risk-Based Security**: ML-powered fraud detection and adaptive multi-factor authentication
- **Compliance & Audit**: SOC 2, HIPAA, PCI-DSS, and GDPR compliance with immutable audit trails

### Target Use Cases

1. **Enterprise SSO**: Centralized authentication for SaaS applications, internal tools, and third-party services
2. **B2B Multi-Tenant SaaS**: Isolated authentication domains for different organizations
3. **API Gateway Authentication**: Secure API access with JWT-based token validation
4. **Mobile Application Backend**: Native authentication flows for iOS and Android apps
5. **Microservices Security**: Service-to-service authentication and authorization

## High-Level Architecture

The platform follows a **microservices-oriented, layered architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                     HTTP API Layer                          │
│              (auth-api, auth-protocols)                     │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  Application Services                       │
│     (Identity, Authorization, Risk Assessment, RBAC)        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    Domain Models                            │
│        (User, Token, Session, Role, Permission)             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Infrastructure Layer                           │
│  (Database, Cache, Crypto, Telemetry, Audit)               │
└─────────────────────────────────────────────────────────────┘
```

### Architectural Layers

#### 1. Domain Layer (`auth-core/models`)
Pure business entities with no external dependencies. Contains:
- User, Token, Session, Role, Permission, Tenant, Organization models
- Business validation rules
- Domain-specific types and enums

#### 2. Application Layer (`auth-core/services`)
Business logic orchestration and use case implementation:
- **IdentityService**: User registration, authentication, lifecycle management
- **AuthorizationService**: RBAC/ABAC permission checking
- **TokenService**: JWT generation, validation, refresh, revocation
- **SessionService**: Session creation, validation, fingerprinting
- **RiskEngine**: Fraud detection, adaptive MFA triggering
- **SubscriptionService**: Tenant subscription tier management

#### 3. Adapter Layer
External interface implementations:
- **auth-api**: HTTP REST API with Axum framework
- **auth-protocols**: SAML, OIDC, OAuth, SCIM protocol handlers
- **auth-db**: Database persistence with SQLx (MySQL/SQLite)
- **auth-extension**: GraphQL API and Rhai scripting engine

#### 4. Infrastructure Layer
Cross-cutting technical concerns:
- **auth-config**: Dynamic configuration management
- **auth-crypto**: Cryptographic operations (Argon2id, JWT, key management)
- **auth-cache**: Multi-layer caching (in-memory + Redis)
- **auth-telemetry**: OpenTelemetry, Prometheus metrics, distributed tracing
- **auth-audit**: Compliance logging, audit trails, event storage

### Crate Structure

The platform is organized as a Cargo workspace with 11 crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `auth-platform` | Binary | Main application orchestrator |
| `auth-core` | Library | Pure business logic (domain + application layers) |
| `auth-api` | Library | HTTP API layer with Axum |
| `auth-protocols` | Library | SAML, OIDC, OAuth, SCIM implementations |
| `auth-db` | Library | Database layer with repository pattern |
| `auth-config` | Library | Configuration loading and management |
| `auth-crypto` | Library | Cryptographic operations and key management |
| `auth-cache` | Library | Caching layer (Redis + in-memory) |
| `auth-telemetry` | Library | Observability and metrics |
| `auth-audit` | Library | Audit logging and compliance |
| `auth-extension` | Library | GraphQL API and scripting extensions |

### Dependency Flow

The architecture enforces strict dependency rules:

```
Infrastructure ← Adapter ← Application ← Domain
```

- **Domain** has no dependencies on other layers
- **Application** depends only on Domain
- **Adapters** depend on Application and Domain
- **Infrastructure** supports all layers but doesn't create circular dependencies

## Why Rust?

Rust was chosen for this security-critical authentication platform for several compelling reasons:

### 1. Memory Safety Without Garbage Collection
- **Zero-cost abstractions**: High-level code compiles to efficient machine code
- **No null pointer exceptions**: Option/Result types enforce explicit error handling
- **No data races**: Ownership system prevents concurrent access bugs at compile time
- **No buffer overflows**: Bounds checking prevents common security vulnerabilities

### 2. Performance
- **Native performance**: Comparable to C/C++ without the safety risks
- **Predictable latency**: No garbage collection pauses affecting response times
- **Efficient concurrency**: Tokio async runtime enables 100k+ concurrent connections
- **Small memory footprint**: Critical for high-density deployments

### 3. Type Safety for Security
- **Compile-time guarantees**: Many security bugs caught before deployment
- **Strong type system**: Prevents type confusion attacks
- **Trait-based abstractions**: Secure, testable interfaces for cryptographic operations
- **No implicit conversions**: Explicit handling of sensitive data transformations

### 4. Ecosystem for Authentication
- **argon2**: Industry-standard password hashing
- **jsonwebtoken**: Secure JWT implementation
- **sqlx**: Compile-time checked SQL queries (prevents SQL injection)
- **rustls**: Memory-safe TLS 1.3 implementation
- **webauthn-rs**: Passwordless authentication support

### 5. Maintainability
- **Fearless refactoring**: Compiler catches breaking changes
- **Excellent tooling**: Cargo, rustfmt, clippy for code quality
- **Documentation culture**: Built-in documentation generation
- **Property-based testing**: Proptest for comprehensive test coverage

## Security Posture

The platform implements defense-in-depth security across multiple layers:

### Authentication Security

#### Password Security
- **Hashing**: Argon2id with per-user salts (OWASP recommended)
- **Complexity**: Minimum 8 characters, configurable policies
- **Common password blacklist**: Prevents use of compromised passwords
- **Password change tracking**: Monitors password_changed_at for security audits

#### Token Security
- **Algorithm**: RS256 (RSA with SHA-256) for JWT signing
- **Short expiration**: 15-minute access tokens, 7-day refresh tokens
- **Token rotation**: Refresh tokens are single-use with rotation
- **Revocation list**: Blacklist for compromised tokens
- **Secure storage**: Refresh tokens hashed before database storage

#### Multi-Factor Authentication (MFA)
- **TOTP support**: Time-based one-time passwords (RFC 6238)
- **Backup codes**: Encrypted recovery codes for account access
- **Adaptive MFA**: Risk-based triggering based on behavioral analysis
- **WebAuthn support**: Hardware security key and biometric authentication

### Network Security

#### Transport Layer
- **TLS 1.3 only**: Modern cipher suites, no legacy protocol support
- **Certificate pinning**: Optional for high-security deployments
- **HSTS headers**: Enforce HTTPS connections

#### Request Security
- **Rate limiting**: 5 requests/minute per IP (configurable)
- **Account lockout**: Automatic lockout after 5 failed login attempts
- **CSRF protection**: Token-based protection for state-changing operations
- **Request ID tracking**: UUID-based correlation for security investigations

#### Security Headers (OWASP Best Practices)
- **Content-Security-Policy**: Prevents XSS attacks
- **X-Frame-Options**: Prevents clickjacking
- **X-Content-Type-Options**: Prevents MIME sniffing
- **Strict-Transport-Security**: Enforces HTTPS
- **Referrer-Policy**: Controls referrer information leakage

### Data Security

#### Secrets Management
- **Environment variables**: Secrets never hardcoded
- **secrecy crate**: Prevents accidental secret logging
- **No secrets in logs**: Structured logging filters sensitive data
- **Key rotation**: Support for cryptographic key rotation

#### Database Security
- **Parameterized queries**: SQLx compile-time checked queries prevent SQL injection
- **Tenant isolation**: Row-level security for multi-tenant data
- **Encrypted connections**: TLS for database connections
- **Audit logging**: All data access logged for compliance

### Application Security

#### Input Validation
- **Email validation**: RFC-compliant with automatic normalization
- **Type-safe parsing**: Rust's type system prevents injection attacks
- **Validator crate**: Declarative validation rules on models
- **Length limits**: Prevents resource exhaustion attacks

#### Error Handling
- **No information leakage**: Generic error messages to clients
- **Detailed internal logging**: Full error context for debugging
- **Request ID correlation**: Track errors across distributed systems

### Operational Security

#### Audit & Compliance
- **Immutable audit logs**: Tamper-evident event logging
- **Comprehensive event tracking**: Login, logout, permission changes, data access
- **Compliance frameworks**: SOC 2, HIPAA, PCI-DSS, GDPR support
- **Audit export**: JSON/CSV export for external analysis

#### Monitoring & Alerting
- **Prometheus metrics**: Real-time security metrics
- **OpenTelemetry tracing**: Distributed request tracing
- **Anomaly detection**: Unusual login patterns, failed attempts
- **Security dashboards**: Real-time visibility into authentication events

## Intended Consumers

### 1. Frontend Web Applications
- **SPA Integration**: React, Vue, Angular applications
- **Authentication Flow**: Authorization Code flow with PKCE
- **Token Management**: Automatic refresh token rotation
- **Session Management**: Secure cookie-based sessions

### 2. Mobile Applications
- **Native Apps**: iOS (Swift) and Android (Kotlin) SDKs
- **OAuth 2.1**: Mobile-optimized authentication flows
- **Biometric Integration**: WebAuthn for fingerprint/Face ID
- **Offline Support**: Cached credentials with secure storage

### 3. Backend Services & APIs
- **Service-to-Service**: JWT validation for microservices
- **API Gateway Integration**: Token introspection endpoints
- **Machine-to-Machine**: Client credentials flow for automated systems
- **GraphQL Support**: Authentication for GraphQL APIs

### 4. Third-Party Integrations
- **SAML 2.0**: Enterprise SSO for legacy applications
- **SCIM 2.0**: Automated user provisioning
- **OIDC**: Standard-compliant identity provider
- **Webhooks**: Real-time event notifications

### 5. Enterprise Identity Providers
- **Identity Federation**: Trust relationships with external IdPs
- **Directory Integration**: LDAP/Active Directory sync
- **Social Login**: Google, Microsoft, GitHub OAuth providers

## Performance Characteristics

### Scalability
- **Concurrent Users**: 100,000 - 1,000,000+ simultaneous sessions
- **Request Throughput**: 10,000+ requests/second per instance
- **Horizontal Scaling**: Stateless design enables unlimited scaling
- **Database Sharding**: Support for multi-region deployments

### Latency
- **Authentication**: Sub-50ms response time (p95)
- **Token Validation**: Sub-10ms (cached)
- **Database Queries**: Optimized indexes for <20ms queries

### Availability
- **Uptime SLA**: 99.99% (52 minutes downtime/year)
- **Automated Failover**: Multi-region active-active deployment
- **Health Checks**: Liveness and readiness probes
- **Graceful Degradation**: Cached authentication during database outages

### Caching Strategy
- **L1 Cache**: In-memory (DashMap) for hot data
- **L2 Cache**: Redis for distributed caching
- **L3 Cache**: CDN for static assets
- **Cache Invalidation**: Real-time updates on data changes

## Compliance & Standards

### Regulatory Compliance
- **SOC 2 Type II**: Security, availability, confidentiality controls
- **HIPAA**: Healthcare data protection (PHI handling)
- **PCI-DSS**: Payment card industry security standards
- **GDPR**: European data protection and privacy rights

### Industry Standards
- **OWASP Top 10**: Protection against common web vulnerabilities
- **NIST Cybersecurity Framework**: Security controls alignment
- **ISO 27001**: Information security management
- **OAuth 2.1**: Latest OAuth security best practices

## Documentation Structure

This documentation is organized as follows:

- **[Code Documentation](./code/README.md)**: Detailed technical documentation
  - **[Crate Documentation](./code/crates.md)**: All 11 crates explained
  - **[Directory Structure](./code/structure.md)**: Complete file tree and organization
  - **[Execution Flows](./code/flows.md)**: Step-by-step authentication flows
  - **[Security Documentation](./code/security.md)**: Security boundaries and threat model
  - **[Dependencies](./code/dependencies.md)**: Internal and external dependencies
  - **[Configuration](./code/configuration.md)**: Runtime configuration and environment variables
  - **[File-by-File Documentation](./code/crates/)**: Detailed documentation for all 109 Rust files

## Getting Started

For development setup, deployment instructions, and API documentation, see:

- [Main README](../README.md): Quick start guide
- [API Reference](./06_api_contracts/auth_flows.md): API endpoint documentation
- [Production Readiness](./07_operations/production_readiness.md): Deployment guide

## Support & Contribution

For technical questions, security reports, or contribution guidelines:

- **Issues**: Create an issue in the repository
- **Security**: Report vulnerabilities privately to the security team
- **Documentation**: All code is documented with inline comments and rustdoc

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Maintained By**: Engineering Team
