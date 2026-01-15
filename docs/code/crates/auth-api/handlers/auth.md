# handlers/auth.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/auth.rs`  
**Crate**: `auth-api`  
**Module**: `handlers::auth`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - Authentication endpoints exposed to public internet

## Purpose

Implements HTTP handlers for authentication endpoints (login, registration) using Axum web framework with OpenAPI documentation.

### Problem It Solves

- Exposes authentication services via REST API
- Validates and sanitizes HTTP requests
- Converts business errors to appropriate HTTP responses
- Provides structured logging for all authentication attempts

---

## Detailed Code Breakdown

### Function: `login()`

**Signature**:
```rust
pub async fn login(
    State(state): State<AppState>,
    Extension(request_id): Extension<Uuid>,
    Json(mut payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError>
```

**Purpose**: Authenticate user and issue tokens

**HTTP Method**: `POST`  
**Path**: `/auth/login`  
**OpenAPI**: Documented with `#[utoipa::path]` macro

**Request Body**: `AuthRequest`
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0..."
}
```

**Response**: `AuthResponse` (200 OK)
```json
{
  "user": { ... },
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "a1b2c3d4e5f6...",
  "requires_mfa": false
}
```

**Error Responses**:
- `401 Unauthorized`: Invalid credentials
- `423 Locked`: Account locked
- `429 Too Many Requests`: Rate limit exceeded

---

### Implementation Steps

#### 1. Email Normalization
```rust
payload.email = validation::validate_email(&payload.email)
    .map_err(|e| ApiError::new(e).with_request_id(request_id))?;
```

**Purpose**: Normalize email format (lowercase, trim whitespace)

**Validation**:
- RFC 5322 email format
- Maximum length check
- Prevents case-sensitivity issues

#### 2. Structured Logging (Request)
```rust
info!(
    request_id = %request_id,
    email = %payload.email,
    "Login attempt"
);
```

**Fields Logged**:
- `request_id`: Unique request identifier (for tracing)
- `email`: User's email (for audit)
- Message: "Login attempt"

**Security**: Password never logged

#### 3. Service Invocation
```rust
match state.identity_service.login(payload.clone()).await {
    Ok(response) => { /* success */ }
    Err(e) => { /* error */ }
}
```

**Delegation**: Business logic handled by `IdentityService`

#### 4. Success Logging
```rust
info!(
    request_id = %request_id,
    email = %payload.email,
    "Login successful"
);
```

#### 5. Error Logging
```rust
warn!(
    request_id = %request_id,
    email = %payload.email,
    error = ?e,
    "Login failed"
);
```

**Level**: `warn` (not `error`) - failed logins are expected

#### 6. Response
```rust
Ok(Json(response))  // 200 OK
// or
Err(ApiError::new(e).with_request_id(request_id))  // Error status
```

---

### Function: `register()`

**Signature**:
```rust
pub async fn register(
    State(state): State<AppState>,
    Extension(request_id): Extension<Uuid>,
    Json(mut payload): Json<CreateUserRequest>,
) -> Result<Json<User>, ApiError>
```

**Purpose**: Register new user account

**HTTP Method**: `POST`  
**Path**: `/auth/register`

**Request Body**: `CreateUserRequest`
```json
{
  "email": "newuser@example.com",
  "password": "SecurePassword123",
  "phone": "+1234567890",
  "profile_data": {
    "first_name": "John",
    "last_name": "Doe"
  }
}
```

**Response**: `User` (200 OK)
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "newuser@example.com",
  "status": "PendingVerification",
  ...
}
```

**Error Responses**:
- `400 Bad Request`: Validation error
- `409 Conflict`: Email already exists
- `429 Too Many Requests`: Rate limit exceeded

---

### Implementation Steps

#### 1. Email Validation
```rust
payload.email = validation::validate_email(&payload.email)
    .map_err(|e| ApiError::new(e).with_request_id(request_id))?;
```

#### 2. Password Validation
```rust
if let Some(ref password) = payload.password {
    validation::validate_password(password)
        .map_err(|e| ApiError::new(e).with_request_id(request_id))?;
} else {
    return Err(ApiError::new(AuthError::ValidationError {
        message: "Password is required".to_string(),
    }).with_request_id(request_id));
}
```

**Password Policy**:
- Minimum 8 characters
- Maximum 128 characters
- Configurable complexity requirements

#### 3. Logging
```rust
info!(
    request_id = %request_id,
    email = %payload.email,
    "Registration attempt"
);
```

#### 4. Tenant ID Extraction
```rust
// TODO: Extract tenant_id from Host header or specialized middleware
let tenant_id = Uuid::default();
```

**Current**: Uses default tenant  
**Future**: Extract from subdomain or header

#### 5. Service Invocation
```rust
match state.identity_service.register(payload.clone(), tenant_id).await {
    Ok(user) => { /* success */ }
    Err(e) => { /* error */ }
}
```

#### 6. Success Logging
```rust
info!(
    request_id = %request_id,
    email = %payload.email,
    user_id = %user.id,
    "Registration successful"
);
```

#### 7. Response
```rust
Ok(Json(user))  // 200 OK with created user
```

---

## OpenAPI Documentation

### Login Endpoint

```rust
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 423, description = "Account locked"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "Authentication"
)]
```

**Generated**: OpenAPI 3.0 specification  
**Accessible**: `/swagger-ui` endpoint

### Register Endpoint

```rust
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Registration successful", body = User),
        (status = 409, description = "Email already exists"),
        (status = 400, description = "Validation error"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "Authentication"
)]
```

---

## Request ID Pattern

### Purpose

Unique identifier for request tracing across services.

### Implementation

**Middleware**: Generates UUID for each request

```rust
// middleware/request_id.rs
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4();
    request.extensions_mut().insert(request_id);
    
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.to_string().parse().unwrap()
    );
    response
}
```

**Usage in Handler**:
```rust
Extension(request_id): Extension<Uuid>
```

**Benefits**:
- Distributed tracing
- Log correlation
- Debugging support

---

## Error Handling

### ApiError Conversion

```rust
impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid credentials".to_string(),
                request_id: None,
            },
            AuthError::Conflict { message } => ApiError {
                status: StatusCode::CONFLICT,
                message,
                request_id: None,
            },
            // ... other conversions
        }
    }
}
```

### HTTP Status Mapping

| AuthError | HTTP Status | Response Body |
|-----------|-------------|---------------|
| `InvalidCredentials` | 401 Unauthorized | `{"error": "Invalid credentials"}` |
| `AccountLocked` | 423 Locked | `{"error": "Account locked: ..."}` |
| `Conflict` | 409 Conflict | `{"error": "Email already registered"}` |
| `ValidationError` | 400 Bad Request | `{"error": "Validation error: ..."}` |
| `RateLimitExceeded` | 429 Too Many Requests | `{"error": "Rate limit exceeded"}` |

---

## Security Considerations

### Input Validation

**Email Normalization**:
```rust
pub fn validate_email(email: &str) -> Result<String, AuthError> {
    let email = email.trim().to_lowercase();
    
    // RFC 5322 validation
    if !email_regex().is_match(&email) {
        return Err(AuthError::ValidationError {
            message: "Invalid email format".to_string()
        });
    }
    
    // Length check
    if email.len() > 255 {
        return Err(AuthError::ValidationError {
            message: "Email too long".to_string()
        });
    }
    
    Ok(email)
}
```

**Password Validation**:
```rust
pub fn validate_password(password: &str) -> Result<(), AuthError> {
    if password.len() < 8 {
        return Err(AuthError::PasswordPolicyViolation {
            errors: vec!["Password must be at least 8 characters".to_string()]
        });
    }
    
    if password.len() > 128 {
        return Err(AuthError::PasswordPolicyViolation {
            errors: vec!["Password too long".to_string()]
        });
    }
    
    // Additional checks (uppercase, lowercase, numbers, special chars)
    // ...
    
    Ok(())
}
```

### Logging Security

**Never Log**:
- Passwords (plaintext or hashed)
- Tokens (access or refresh)
- MFA secrets
- Backup codes

**Safe to Log**:
- Email addresses (for audit)
- Request IDs
- IP addresses
- User agents
- Error types (not details)

### Rate Limiting

**Applied by Middleware**: Before handler execution

```rust
// middleware/rate_limit.rs
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let ip = extract_ip(&request);
    let endpoint = request.uri().path();
    
    if is_rate_limited(&ip, endpoint).await? {
        return Err(ApiError::new(AuthError::RateLimitExceeded {
            limit: 5,
            window: "minute".to_string(),
        }));
    }
    
    Ok(next.run(request).await)
}
```

**Limits**:
- Login: 5 attempts/minute per IP
- Registration: 3 attempts/hour per IP

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `serde` | JSON serialization |
| `uuid` | Request IDs |
| `tracing` | Structured logging |
| `utoipa` | OpenAPI documentation |

### Internal Dependencies

- [auth-core/services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - IdentityService
- [auth-core/models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User, CreateUserRequest
- [error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/error.rs) - ApiError
- [validation.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/validation.rs) - Input validation

---

## Testing

### Integration Tests

```rust
#[tokio::test]
async fn test_login_success() {
    let app = test_app().await;
    
    let response = app
        .post("/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123",
            "tenant_id": "00000000-0000-0000-0000-000000000000"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    let body: AuthResponse = response.json().await;
    assert!(!body.access_token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let app = test_app().await;
    
    let response = app
        .post("/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "wrong_password",
            "tenant_id": "00000000-0000-0000-0000-000000000000"
        }))
        .send()
        .await;
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
```

---

## Related Files

- [router.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/router.rs) - Route registration
- [middleware/request_id.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/request_id.rs) - Request ID generation
- [middleware/rate_limit.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/rate_limit.rs) - Rate limiting
- [validation.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/validation.rs) - Input validation

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 122  
**Security Level**: CRITICAL
