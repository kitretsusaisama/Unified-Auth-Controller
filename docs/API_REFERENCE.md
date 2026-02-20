# API Reference

This document provides a comprehensive reference for all API endpoints. For interactive documentation, visit the **Swagger UI** at `/swagger-ui` when the server is running.

## Base URL

```
http://localhost:8081
```

## Interactive Documentation

**Swagger UI**: [http://localhost:8081/swagger-ui](http://localhost:8081/swagger-ui)

Explore and test all API endpoints interactively with auto-generated request/response examples.

## Authentication

Most endpoints require authentication via JWT tokens. Include the token in the `Authorization` header:

```
Authorization: Bearer <access_token>
```

## Request Tracking

All requests receive a unique `x-request-id` header in the response for distributed tracing:

```
x-request-id: 550e8400-e29b-41d4-a716-446655440000
```

Include this ID when reporting issues for faster troubleshooting.

## Rate Limiting

**Limit**: 5 requests per minute per IP address

**Headers**:
- Rate limit status is indicated via HTTP 429 (Too Many Requests)
- Retry after 60 seconds when rate limited

## Common Response Codes

- `200 OK` - Request successful
- `400 Bad Request` - Invalid request parameters or validation error
- `401 Unauthorized` - Missing or invalid authentication
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists (e.g., duplicate email)
- `423 Locked` - Account is locked
- `429 Too Many Requests` - Rate limit exceeded
- `500 Internal Server Error` - Server error

---

## Error Response Format

All errors return a structured JSON response with error codes and request IDs:

```json
{
  "code": "ERROR_CODE",
  "message": "Human-readable error message",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "fields": [
    {
      "field": "password",
      "message": "Password must be at least 12 characters long"
    }
  ]
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_CREDENTIALS` | 401 | Email or password is incorrect |
| `UNAUTHORIZED` | 403 | Insufficient permissions |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `PASSWORD_POLICY_VIOLATION` | 400 | Password doesn't meet requirements |
| `USER_NOT_FOUND` | 404 | User does not exist |
| `ACCOUNT_LOCKED` | 423 | Account is locked due to failed attempts |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `CONFLICT` | 409 | Resource already exists |
| `TOKEN_ERROR` | 401 | Invalid, expired, or revoked token |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

### Password Policy

Passwords must meet the following requirements:
- **Minimum length**: 12 characters
- **Maximum length**: 128 characters
- **Complexity**:
  - At least one uppercase letter (A-Z)
  - At least one lowercase letter (a-z)
  - At least one number (0-9)
  - At least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)
- **Not in common password list**: Rejects 24+ most common passwords

### Email Validation

- Must be valid RFC-compliant email format
- Automatically normalized to lowercase
- Whitespace trimmed
- Maximum 254 characters

---

## Endpoints

### Health Check

#### `GET /health`

Check if the service is running and healthy.

**Response:**
```json
{
  "status": "ok",
  "message": "SSO Platform API is healthy",
  "version": "0.1.0"
}
```

---

### Authentication

#### `POST /auth/register`

Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "phone": "+1234567890",
  "profile_data": {
    "first_name": "John",
    "last_name": "Doe"
  }
}
```

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "email_verified": false,
  "status": "pending_verification",
  "created_at": "2026-01-11T10:00:00Z",
  ...
}
```

**Error Responses:**
- `409 Conflict`: Email already exists
- `400 Bad Request`: Validation error (e.g., weak password)

---

#### `POST /auth/login`

Authenticate a user and receive access/refresh tokens.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Response (200 OK):**
```json
{
  "user": { ... },
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "a1b2c3d4e5f6...",
  "requires_mfa": false
}
```

**Error Responses:**
- `401 Unauthorized`: Invalid credentials
- `423 Locked`: Account locked due to failed login attempts

---

### User Management

#### `POST /users/{id}/ban`

Suspend a user account (Admin only).

**Path Parameters:**
- `id` (UUID): User ID to ban

**Response (200 OK):**
```json
{
  "status": "success",
  "message": "User suspended"
}
```

**Error Responses:**
- `404 Not Found`: User not found
- `403 Forbidden`: Insufficient permissions

---

#### `POST /users/{id}/activate`

Activate a suspended user account (Admin only).

**Path Parameters:**
- `id` (UUID): User ID to activate

**Response (200 OK):**
```json
{
  "status": "success",
  "message": "User activated"
}
```

---

### OIDC/OAuth 2.1

#### `GET /auth/oidc/login`

Initiate OIDC login flow (redirects to external IdP).

**Query Parameters:**
- `provider` (string): OIDC provider name (e.g., "google", "okta")

---

#### `GET /auth/oidc/callback`

OIDC callback endpoint (handles authorization code exchange).

**Query Parameters:**
- `code` (string): Authorization code from IdP
- `state` (string): CSRF protection token

---

### SAML 2.0

#### `GET /auth/saml/metadata`

Retrieve SAML Service Provider metadata XML.

**Response:** XML document

---

#### `POST /auth/saml/acs`

SAML Assertion Consumer Service (ACS) endpoint.

**Request Body:** SAML Response (form-encoded)

---

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_CREDENTIALS` | 401 | Invalid email or password |
| `ACCOUNT_LOCKED` | 423 | Too many failed login attempts |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `RESOURCE_NOT_FOUND` | 404 | Requested resource doesn't exist |
| `CONFLICT` | 409 | Resource already exists |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Authentication endpoints**: 5 requests per minute per IP
- **General endpoints**: 100 requests per minute per user

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1641900000
```

## Pagination

List endpoints support pagination using query parameters:

```http
GET /users?page=1&per_page=50
```

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 50,
    "total": 1250,
    "total_pages": 25
  }
}
```

## Interactive Documentation

For a complete, interactive API reference with request/response examples, visit:

```
http://localhost:8080/swagger-ui
```

This Swagger UI interface allows you to:
- Explore all endpoints
- View detailed request/response schemas
- Test API calls directly from the browser
- Download the OpenAPI specification
