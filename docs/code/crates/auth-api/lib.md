# lib.rs (auth-api)

## File Metadata

**File Path**: `crates/auth-api/src/lib.rs`  
**Crate**: `auth-api`  
**Module**: Root  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - HTTP API entry point

## Purpose

Root module for the `auth-api` crate, defining the HTTP API layer with Axum web framework, OpenAPI documentation, and application state management.

### Problem It Solves

- Exposes authentication services via REST API
- Provides OpenAPI/Swagger documentation
- Manages application state (services, database connections)
- Integrates middleware for security and observability

---

## Detailed Code Breakdown

### Module Structure

```rust
pub mod router;      // Route definitions
pub mod handlers;    // HTTP request handlers
pub mod error;       // HTTP error responses
pub mod validation;  // Input validation
pub mod middleware;  // HTTP middleware
```

**Architecture**: Layered HTTP adapter over business logic

---

### Struct: `ApiDoc`

**Purpose**: OpenAPI documentation specification

**Implementation**:
```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::login,
        handlers::auth::register,
        handlers::users::ban_user,
        handlers::users::activate_user,
        handlers::health::health_check,
    ),
    components(
        schemas(
            auth_core::services::identity::AuthRequest,
            auth_core::services::identity::AuthResponse,
            auth_core::models::user::User,
            auth_core::models::user::CreateUserRequest,
            auth_core::models::user::UserStatus,
            crate::error::ErrorResponse,
            crate::error::FieldError,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and registration endpoints"),
        (name = "User Management", description = "User administration endpoints"),
        (name = "Health", description = "Service health check endpoints")
    ),
    info(
        title = "Enterprise SSO Platform API",
        version = "0.1.0",
        description = "Production-ready SSO and Identity Platform supporting OIDC, SAML, OAuth 2.1, and SCIM 2.0",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    )
)]
pub struct ApiDoc;
```

**Generated Documentation**:
- **Format**: OpenAPI 3.0 JSON
- **Endpoint**: `/api-docs/openapi.json`
- **UI**: Swagger UI at `/swagger-ui`

**Documented Endpoints**:
1. `POST /auth/login` - User authentication
2. `POST /auth/register` - User registration
3. `POST /users/{id}/ban` - Ban user account
4. `POST /users/{id}/activate` - Activate user account
5. `GET /health` - Health check

**Documented Schemas**:
- `AuthRequest` - Login request body
- `AuthResponse` - Login response
- `User` - User entity
- `CreateUserRequest` - Registration request
- `UserStatus` - User status enum
- `ErrorResponse` - Error response format
- `FieldError` - Validation error details

---

### Struct: `AppState`

**Purpose**: Shared application state passed to all handlers

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `db` | `MySqlPool` | Database connection pool |
| `role_service` | `Arc<RoleService>` | Role management service |
| `session_service` | `Arc<SessionService>` | Session management service |
| `subscription_service` | `Arc<SubscriptionService>` | Subscription service |
| `identity_service` | `Arc<IdentityService>` | Authentication service |

**Pattern**: Dependency injection via Axum state

**Usage in Handlers**:
```rust
pub async fn login(
    State(state): State<AppState>,  // Injected by Axum
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Access services via state
    let response = state.identity_service.login(payload).await?;
    Ok(Json(response))
}
```

**Thread Safety**: All fields are `Arc`-wrapped for cheap cloning across threads

---

### Function: `app()`

**Signature**: `pub fn app(state: AppState) -> Router`

**Purpose**: Constructs the Axum application router

**Implementation**:
```rust
pub fn app(state: AppState) -> Router {
    router::api_router()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
```

**Components**:

1. **API Router**: Main application routes
   ```rust
   router::api_router()
   ```
   - Defined in `router.rs`
   - Includes all API endpoints
   - Applies middleware

2. **Swagger UI**: Interactive API documentation
   ```rust
   .merge(SwaggerUi::new("/swagger-ui")
       .url("/api-docs/openapi.json", ApiDoc::openapi()))
   ```
   - **UI Endpoint**: `http://localhost:8080/swagger-ui`
   - **JSON Spec**: `http://localhost:8080/api-docs/openapi.json`
   - **Features**: Try-it-out, schema explorer, authentication

3. **State Injection**: Shares state with all handlers
   ```rust
   .with_state(state)
   ```
   - Makes `AppState` available via `State` extractor
   - Cloned for each request (cheap with `Arc`)

**Returns**: Configured `Router` ready to serve requests

---

## Application Initialization

### Typical Setup (from main.rs)

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize database
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    
    // 2. Initialize repositories
    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let role_repo = Arc::new(RoleRepository::new(pool.clone()));
    let session_repo = Arc::new(SessionRepository::new(pool.clone()));
    let subscription_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
    
    // 3. Initialize services
    let token_service = Arc::new(TokenEngine::new().await?);
    let identity_service = Arc::new(IdentityService::new(
        user_repo,
        token_service,
    ));
    let role_service = Arc::new(RoleService::new(role_repo));
    let session_service = Arc::new(SessionService::new(session_repo));
    let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));
    
    // 4. Create application state
    let app_state = AppState {
        db: pool,
        role_service,
        session_service,
        subscription_service,
        identity_service,
    };
    
    // 5. Build application
    let app = auth_api::app(app_state);
    
    // 6. Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

---

## OpenAPI Features

### Swagger UI

**Access**: `http://localhost:8080/swagger-ui`

**Features**:
- **Interactive Testing**: Try API endpoints directly from browser
- **Schema Explorer**: View request/response schemas
- **Authentication**: Test with Bearer tokens
- **Code Generation**: Generate client SDKs

**Example Usage**:
1. Navigate to `/swagger-ui`
2. Click "Authorize" button
3. Enter Bearer token: `eyJhbGciOiJSUzI1NiIs...`
4. Try `POST /auth/login` endpoint
5. View response

### OpenAPI JSON

**Access**: `http://localhost:8080/api-docs/openapi.json`

**Use Cases**:
- **Client Generation**: Generate TypeScript, Python, Java clients
- **API Gateway**: Import into Kong, AWS API Gateway
- **Testing**: Use with Postman, Insomnia
- **Documentation**: Generate static docs with Redoc

**Example**:
```bash
# Generate TypeScript client
npx @openapitools/openapi-generator-cli generate \
  -i http://localhost:8080/api-docs/openapi.json \
  -g typescript-axios \
  -o ./client
```

---

## Middleware Stack

### Applied Middleware (from router.rs)

```rust
pub fn api_router() -> Router<AppState> {
    Router::new()
        // Routes
        .route("/auth/login", post(handlers::auth::login))
        .route("/auth/register", post(handlers::auth::register))
        // ...
        
        // Middleware (applied in reverse order)
        .layer(middleware::request_id::request_id_middleware())
        .layer(middleware::security_headers::security_headers_middleware())
        .layer(middleware::rate_limit::rate_limit_middleware())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive())
}
```

**Execution Order** (request → response):
1. CORS
2. Tracing
3. Rate limiting
4. Security headers
5. Request ID
6. Handler

---

## Security Features

### 1. Security Headers

**Middleware**: `middleware/security_headers.rs`

**Headers Applied**:
```
Content-Security-Policy: default-src 'self'; ...
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Strict-Transport-Security: max-age=31536000; includeSubDomains
```

### 2. Rate Limiting

**Middleware**: `middleware/rate_limit.rs`

**Limits**:
- Login: 5 attempts/minute per IP
- Registration: 3 attempts/hour per IP
- API calls: 1000 requests/hour per user

### 3. Request ID

**Middleware**: `middleware/request_id.rs`

**Purpose**: Distributed tracing

**Header**: `X-Request-ID: 550e8400-e29b-41d4-a716-446655440000`

### 4. CORS

**Configuration**: Permissive (development)

**Production**:
```rust
CorsLayer::new()
    .allow_origin("https://app.example.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
```

---

## Error Handling

### Error Response Format

**Defined in**: `error.rs`

```rust
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub request_id: Option<Uuid>,
    pub fields: Option<Vec<FieldError>>,
}

#[derive(Serialize, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}
```

**Example Response**:
```json
{
  "error": "ValidationError",
  "message": "Invalid input",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
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

## Testing

### Integration Tests

```rust
#[tokio::test]
async fn test_login_endpoint() {
    let app_state = create_test_app_state().await;
    let app = auth_api::app(app_state);
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"email":"test@example.com","password":"password123"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `sqlx` | Database connection pool |
| `utoipa` | OpenAPI documentation |
| `utoipa-swagger-ui` | Swagger UI |
| `tower-http` | HTTP middleware |

### Internal Dependencies

- [auth-core](file:///c:/Users/Victo/Downloads/sso/crates/auth-core) - Business logic
- [auth-db](file:///c:/Users/Victo/Downloads/sso/crates/auth-db) - Database layer

---

## Related Files

- [router.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/router.rs) - Route definitions
- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Auth handlers
- [error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/error.rs) - Error types
- [main.rs](file:///c:/Users/Victo/Downloads/sso/src/main.rs) - Application entry point

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 75  
**Security Level**: CRITICAL
