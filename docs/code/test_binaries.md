# Test Binaries Documentation

## Overview

The `src/bin/` directory contains 16 integration test binaries that validate different aspects of the authentication platform. Each binary is a standalone executable that tests specific functionality.

---

## Test Binaries

### 1. test_anomaly.rs
**Purpose**: Tests anomaly detection system  
**Tests**: Behavioral anomaly detection, risk scoring

### 2. test_audit.rs
**Purpose**: Tests audit logging system  
**Tests**: Audit event creation, storage, retrieval

### 3. test_audit_export.rs
**Purpose**: Tests audit log export functionality  
**Tests**: Compliance reporting, log export formats

### 4. test_auth_flow.rs
**Purpose**: Tests complete authentication flow  
**Tests**: Login, registration, token generation, session creation

### 5. test_caching.rs
**Purpose**: Tests multi-level caching  
**Tests**: L1/L2 cache, cache invalidation, TTL

### 6. test_crypto_property.rs
**Purpose**: Property-based testing for cryptography  
**Tests**: Encryption/decryption, hashing properties

### 7. test_extension.rs
**Purpose**: Tests extension mechanisms  
**Tests**: Plugins, webhooks, GraphQL

### 8. test_mysql.rs
**Purpose**: Tests MySQL database operations  
**Tests**: Connection pooling, CRUD operations

### 9. test_oauth.rs
**Purpose**: Tests OAuth 2.0 flow  
**Tests**: Authorization code flow, token exchange

### 10. test_observability.rs
**Purpose**: Tests telemetry and monitoring  
**Tests**: Logging, metrics, tracing

### 11. test_passwordless.rs
**Purpose**: Tests WebAuthn passwordless authentication  
**Tests**: Passkey registration, authentication

### 12. test_protocol_property.rs
**Purpose**: Property-based testing for protocols  
**Tests**: OIDC, SAML, OAuth properties

### 13. test_rbac.rs
**Purpose**: Tests role-based access control  
**Tests**: Role assignment, permission checking, hierarchy

### 14. test_risk_session.rs
**Purpose**: Tests risk assessment and session management  
**Tests**: Risk scoring, session fingerprinting

### 15. test_sharding.rs
**Purpose**: Tests database sharding  
**Tests**: Consistent hashing, shard distribution

### 16. test_subscription.rs
**Purpose**: Tests subscription and feature gating  
**Tests**: Subscription tiers, quota enforcement

---

## Running Tests

```bash
# Run specific test
cargo run --bin test_auth_flow

# Run all tests
for test in src/bin/test_*.rs; do
    cargo run --bin $(basename $test .rs)
done
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Total Test Binaries**: 16  
**Coverage**: Integration testing
