---
title: API Contract Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: API Contracts
---

# API Contract Specification

> [!IMPORTANT]
> **Purpose**: Freeze API interfaces - contracts, not implementation. This is the source of truth for API consumers.

---

## 1. REST API

### 1.1 Base URL

```
Production: https://auth.upflame.com
Staging: https://auth-staging.upflame.com
Development: http://localhost:8080
```

---

### 1.2 Authentication Endpoints

#### POST /auth/register

**Request**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "phone": "+1234567890",
  "profile_data": {
    "first_name": "John",
    "last_name": "Doe"
  }
}
```

**Response (201 Created)**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "status": "pending_verification",
  "created_at": "2026-01-12T10:00:00Z"
}
```

---

#### POST /auth/login

**Request**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "tenant_id": "tenant-uuid"
}
```

**Response (200 OK)**:
```json
{
  "user": { ... },
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "a1b2c3d4e5f6...",
  "requires_mfa": false
}
```

---

#### POST /auth/refresh

**Request**:
```json
{
  "refresh_token": "a1b2c3d4e5f6..."
}
```

**Response (200 OK)**:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "new-refresh-token..."
}
```

---

### 1.3 Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_CREDENTIALS` | 401 | Email or password incorrect |
| `ACCOUNT_LOCKED` | 423 | Account locked due to failed attempts |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `TOKEN_ERROR` | 401 | Invalid, expired, or revoked token |
| `CONFLICT` | 409 | Resource already exists |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |

---

## 2. GraphQL API

### 2.1 Endpoint

```
POST /graphql
```

### 2.2 Schema

```graphql
type User {
  id: ID!
  email: String!
  status: UserStatus!
  createdAt: DateTime!
  roles: [Role!]!
}

type Query {
  me: User!
  user(id: ID!): User
  users(limit: Int, offset: Int): [User!]!
}

type Mutation {
  updateProfile(input: UpdateProfileInput!): User!
  changePassword(oldPassword: String!, newPassword: String!): Boolean!
}
```

---

## 3. Versioning

### 3.1 API Versioning

**Strategy**: URL-based versioning

**Format**: `/v1/auth/login`, `/v2/auth/login`

**Current Version**: v1

### 3.2 Deprecation Policy

- **Notice Period**: 6 months
- **Support Period**: 12 months after deprecation
- **Breaking Changes**: Require new major version

---

## 4. Rate Limiting

### 4.1 Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/auth/login` | 5 requests | 1 minute |
| `/auth/register` | 3 requests | 1 minute |
| `/auth/refresh` | 10 requests | 1 minute |
| Other endpoints | 100 requests | 1 minute |

### 4.2 Headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1641900000
```

---

**Document Status**: Active  
**Owner**: Engineering Team
