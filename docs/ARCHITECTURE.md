# Enterprise SSO Platform Architecture

## Overview
The Enterprise SSO Platform is a hyperscale identity provider supporting OIDC, SAML, and OAuth 2.1 protocols. It is designed for multi-tenant SaaS applications, enforcing strict tenant isolation and scalable authorization (RBAC/ABAC).

## Architecture Components

### 1. Auth Core (`crates/auth-core`)
- **Identity Service**: User management, authentication flows (password, magic link, OTP).
- **Token Engine**: JWT issuance, validation, rotation, and revocation.
- **Workflow Engine**: Universal state machine for multi-step authentication flows.
- **Authorization Service**: Dynamic RBAC/ABAC policy enforcement.

### 2. Auth API (`crates/auth-api`)
- **Axum Router**: High-performance async HTTP handlers.
- **Middleware**: Rate limiting, Audit logging, Request ID tracing.
- **Versioning**: `/v1` namespace for stable API.

### 3. Database (`crates/auth-db`)
- **SQLx Repository**: Type-safe database access.
- **Tenant Isolation**: All queries enforced with `tenant_id`.
- **Migrations**: Versioned, idempotent SQL migrations.

### 4. Protocol Support (`crates/auth-protocols`)
- **OIDC**: Authorization Code Flow with PKCE.
- **SAML**: SP-initiated SSO.
- **OAuth**: Client Credentials, Token Exchange.

## Security Design
- **Secret Management**: Environment-based configuration, no hardcoded secrets.
- **Cryptography**: Argon2id password hashing, RS256 JWT signing with key rotation.
- **Audit**: Async audit logging for compliance.
