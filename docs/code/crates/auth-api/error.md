# error.rs

## File Metadata

**File Path**: `crates/auth-api/src/error.rs`  
**Crate**: `auth-api`  
**Module**: `error`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ⚠️ **MEDIUM** - Error handling and information disclosure

## Purpose

Provides structured error responses for the HTTP API, converting domain errors (`AuthError`) into appropriate HTTP status codes and JSON responses with request tracing.

### Problem It Solves

- Consistent error response format
- Appropriate HTTP status codes
- Request ID correlation
- Information disclosure prevention
- Client-friendly error messages

---

## Detailed Code Breakdown

### Struct: `ErrorResponse`

**Purpose**: Structured JSON error response

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `code` | `String` | Machine-readable error code |
| `message` | `String` | Human-readable message |
| `fields` | `Option<Vec<FieldError>>` | Field-level validation errors |
| `request_id` | `Option<String>` | Request ID for tracing |

**OpenAPI Schema**: Annotated with `#[derive(utoipa::ToSchema)]`

**Example Response**:
```json
{
  "code": "INVALID_CREDENTIALS",
  "message": "The provided credentials are invalid",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Struct: `FieldError`

**Purpose**: Field-level validation error

**Fields**:
- `field`: Field name (e.g., "email")
- `message`: Error message for field

**Example**:
```json
{
  "code": "VALIDATION_ERROR",
  "message": "Validation failed",
  "fields": [
    {
      "field": "email",
      "message": "Invalid email format"
    },
    {
      "field": "password",
      "message": "Password must be at least 8 characters"
    }
  ]
}
```

---

### Struct: `ApiError`

**Purpose**: API error wrapper with request context

**Fields**:
- `inner`: `AuthError` - Domain error
- `request_id`: `Option<Uuid>` - Request ID for tracing

---

### Method: `ApiError::new()`

**Signature**: `pub fn new(error: AuthError) -> Self`

**Purpose**: Create API error from domain error

---

### Method: `ApiError::with_request_id()`

**Signature**: `pub fn with_request_id(mut self, request_id: Uuid) -> Self`

**Purpose**: Add request ID for tracing

**Usage**:
```rust
pub async fn handler(
    Extension(request_id): Extension<Uuid>,
) -> Result<Response, ApiError> {
    some_operation()
        .await
        .map_err(|e| ApiError::new(e).with_request_id(request_id))
}
```

---

## Error Mapping

### Trait Implementation: `IntoResponse for ApiError`

**Purpose**: Convert `ApiError` to HTTP response

**Mapping Table**:

| AuthError | HTTP Status | Error Code |
|-----------|-------------|------------|
| `InvalidCredentials` | 401 UNAUTHORIZED | `INVALID_CREDENTIALS` |
| `Unauthorized` | 403 FORBIDDEN | `UNAUTHORIZED` |
| `Conflict` | 409 CONFLICT | `CONFLICT` |
| `ValidationError` | 400 BAD_REQUEST | `VALIDATION_ERROR` |
| `UserNotFound` | 404 NOT_FOUND | `USER_NOT_FOUND` |
| `AccountLocked` | 423 LOCKED | `ACCOUNT_LOCKED` |
| `RateLimitExceeded` | 429 TOO_MANY_REQUESTS | `RATE_LIMIT_EXCEEDED` |
| `TokenError` | 401 UNAUTHORIZED | `TOKEN_ERROR` |
| `PasswordPolicyViolation` | 400 BAD_REQUEST | `PASSWORD_POLICY_VIOLATION` |
| Others | 500 INTERNAL_SERVER_ERROR | `INTERNAL_ERROR` |

---

### Error Mapping Examples

#### 1. Invalid Credentials
```rust
AuthError::InvalidCredentials => (
    StatusCode::UNAUTHORIZED,
    "INVALID_CREDENTIALS",
    "The provided credentials are invalid".to_string(),
)
```

**Response**:
```json
{
  "code": "INVALID_CREDENTIALS",
  "message": "The provided credentials are invalid",
  "request_id": "..."
}
```

---

#### 2. Account Locked
```rust
AuthError::AccountLocked { reason } => (
    StatusCode::LOCKED,
    "ACCOUNT_LOCKED",
    reason.clone(),
)
```

**Response**:
```json
{
  "code": "ACCOUNT_LOCKED",
  "message": "Account locked due to multiple failed login attempts",
  "request_id": "..."
}
```

---

#### 3. Rate Limit Exceeded
```rust
AuthError::RateLimitExceeded { limit, window } => (
    StatusCode::TOO_MANY_REQUESTS,
    "RATE_LIMIT_EXCEEDED",
    format!("Rate limit exceeded: {} requests per {}", limit, window),
)
```

**Response**:
```json
{
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded: 5 requests per 60s",
  "request_id": "..."
}
```

**Headers**:
```
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1673456789
```

---

#### 4. Password Policy Violation
```rust
AuthError::PasswordPolicyViolation { errors } => (
    StatusCode::BAD_REQUEST,
    "PASSWORD_POLICY_VIOLATION",
    errors.join(", "),
)
```

**Response**:
```json
{
  "code": "PASSWORD_POLICY_VIOLATION",
  "message": "Password must be at least 8 characters, Password must contain uppercase letter",
  "request_id": "..."
}
```

---

## Security Considerations

### 1. Information Disclosure Prevention

**Problem**: Detailed error messages can leak sensitive information

**Solution**: Generic messages for internal errors

```rust
_ => (
    StatusCode::INTERNAL_SERVER_ERROR,
    "INTERNAL_ERROR",
    "An internal error occurred".to_string(), // Generic message
)
```

**Bad Practice**:
```json
{
  "code": "DATABASE_ERROR",
  "message": "Connection to mysql://user:password@localhost:3306/db failed"
}
```

**Good Practice**:
```json
{
  "code": "INTERNAL_ERROR",
  "message": "An internal error occurred",
  "request_id": "550e8400-..."
}
```

---

### 2. User Enumeration Prevention

**Problem**: Different errors for "user not found" vs "wrong password"

**Solution**: Same error for both cases

```rust
// Good: Same error for both cases
AuthError::InvalidCredentials => (
    StatusCode::UNAUTHORIZED,
    "INVALID_CREDENTIALS",
    "The provided credentials are invalid".to_string(),
)

// Bad: Reveals if user exists
AuthError::UserNotFound => "User does not exist"
AuthError::WrongPassword => "Password is incorrect"
```

---

### 3. Request ID for Debugging

**Purpose**: Allow users to report errors without exposing internals

**Usage**:
```rust
let error_response = ErrorResponse {
    code: code.to_string(),
    message,
    fields: None,
    request_id: self.request_id.map(|id| id.to_string()),
};
```

**User Support Flow**:
1. User encounters error
2. User provides request ID to support
3. Support team searches logs by request ID
4. Full error details available in logs (not exposed to user)

---

## Usage Examples

### Example 1: Handler with Error Conversion

```rust
pub async fn login(
    Extension(request_id): Extension<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let response = state
        .identity_service
        .login(payload)
        .await
        .map_err(|e| ApiError::new(e).with_request_id(request_id))?;
    
    Ok(Json(response))
}
```

---

### Example 2: Validation Errors

```rust
pub async fn register(
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<User>, ApiError> {
    // Validate payload
    let mut field_errors = Vec::new();
    
    if !is_valid_email(&payload.email) {
        field_errors.push(FieldError {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        });
    }
    
    if payload.password.len() < 8 {
        field_errors.push(FieldError {
            field: "password".to_string(),
            message: "Password must be at least 8 characters".to_string(),
        });
    }
    
    if !field_errors.is_empty() {
        return Err(ApiError::new(AuthError::ValidationError {
            message: "Validation failed".to_string(),
        }));
    }
    
    // Proceed with registration
    // ...
}
```

---

### Example 3: Custom Error Response

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self.inner {
            // ... error mapping ...
        };
        
        let mut error_response = ErrorResponse {
            code: code.to_string(),
            message,
            fields: None,
            request_id: self.request_id.map(|id| id.to_string()),
        };
        
        // Add field errors for validation failures
        if let AuthError::ValidationError { .. } = &self.inner {
            error_response.fields = Some(extract_field_errors(&self.inner));
        }
        
        (status, Json(error_response)).into_response()
    }
}
```

---

## Testing

### Unit Tests

```rust
#[test]
fn test_invalid_credentials_mapping() {
    let error = ApiError::new(AuthError::InvalidCredentials);
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    
    let body = extract_json_body(response);
    assert_eq!(body["code"], "INVALID_CREDENTIALS");
}

#[test]
fn test_request_id_included() {
    let request_id = Uuid::new_v4();
    let error = ApiError::new(AuthError::InvalidCredentials)
        .with_request_id(request_id);
    
    let response = error.into_response();
    let body = extract_json_body(response);
    
    assert_eq!(body["request_id"], request_id.to_string());
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework, `IntoResponse` |
| `serde` | JSON serialization |
| `uuid` | Request IDs |
| `utoipa` | OpenAPI schema |

### Internal Dependencies

- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [middleware/request_id.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/middleware/request_id.md) - Request ID middleware
- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Auth handlers

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 123  
**Security Level**: MEDIUM
