# middleware/request_id.rs

## File Metadata

**File Path**: `crates/auth-api/src/middleware/request_id.rs`  
**Crate**: `auth-api`  
**Module**: `middleware::request_id`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ⚠️ **MEDIUM** - Request tracing and observability

## Purpose

Axum middleware that generates or preserves unique request IDs for distributed tracing, logging correlation, and debugging across microservices.

### Problem It Solves

- Correlates logs across distributed systems
- Enables request tracing through microservices
- Facilitates debugging of specific requests
- Supports observability and monitoring

---

## Detailed Code Breakdown

### Constant: `REQUEST_ID_HEADER`

```rust
pub const REQUEST_ID_HEADER: &str = "x-request-id";
```

**Purpose**: Standard header name for request ID  
**Convention**: `X-Request-ID` (case-insensitive)

---

### Function: `request_id_middleware()`

**Signature**: `pub async fn request_id_middleware(mut req: Request, next: Next) -> Response`

**Purpose**: Generate or preserve request ID

**Process**:

#### 1. Check Existing Request ID

```rust
let request_id = req
    .headers()
    .get(REQUEST_ID_HEADER)
    .and_then(|v| v.to_str().ok())
    .and_then(|s| Uuid::parse_str(s).ok())
    .unwrap_or_else(Uuid::new_v4);
```

**Logic**:
- If header exists and is valid UUID → use it
- Otherwise → generate new UUID v4

**Why Preserve**:
- Load balancers may set request ID
- API gateways may propagate request ID
- Maintains tracing across services

---

#### 2. Store in Request Extensions

```rust
req.extensions_mut().insert(request_id);
```

**Purpose**: Make request ID available to handlers

**Usage in Handlers**:
```rust
pub async fn my_handler(
    Extension(request_id): Extension<Uuid>,
) -> Response {
    info!("Processing request {}", request_id);
    // ...
}
```

---

#### 3. Add to Response Headers

```rust
let mut response = next.run(req).await;

if let Ok(header_value) = HeaderValue::from_str(&request_id.to_string()) {
    response.headers_mut().insert(REQUEST_ID_HEADER, header_value);
}
```

**Purpose**: Client can correlate request with logs

**Example Response**:
```
HTTP/1.1 200 OK
X-Request-ID: 550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json
```

---

## Integration

### Axum Router Setup

```rust
use axum::{Router, middleware};
use crate::middleware::request_id::request_id_middleware;

pub fn app() -> Router {
    Router::new()
        .route("/api/users", get(list_users))
        .route("/api/auth/login", post(login))
        // Apply request ID middleware to all routes
        .layer(middleware::from_fn(request_id_middleware))
}
```

---

## Logging Integration

### Structured Logging with Request ID

```rust
use tracing::{info, error, Span};
use uuid::Uuid;

pub async fn login(
    Extension(request_id): Extension<Uuid>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Create span with request ID
    let span = tracing::info_span!(
        "login_handler",
        request_id = %request_id,
        email = %payload.email
    );
    
    let _enter = span.enter();
    
    info!("Login attempt");
    
    match identity_service.login(payload).await {
        Ok(response) => {
            info!("Login successful");
            Ok(Json(response))
        }
        Err(e) => {
            error!("Login failed: {}", e);
            Err(e.into())
        }
    }
}
```

**Log Output**:
```json
{
  "timestamp": "2024-01-13T14:05:00Z",
  "level": "INFO",
  "message": "Login attempt",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com"
}
```

---

## Distributed Tracing

### Propagating Request ID Across Services

```rust
pub async fn call_external_service(
    request_id: Uuid,
    url: &str,
) -> Result<Response> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(url)
        .header("X-Request-ID", request_id.to_string())
        .send()
        .await?;
    
    Ok(response)
}
```

**Flow**:
```
Client → API Gateway → Service A → Service B → Database
         [req-123]    [req-123]    [req-123]    [req-123]
```

---

## Error Correlation

### Linking Errors to Requests

```rust
pub async fn handler(
    Extension(request_id): Extension<Uuid>,
) -> Result<Response, ApiError> {
    match risky_operation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "Operation failed"
            );
            
            Err(ApiError::InternalServerError {
                message: format!("Request ID: {}", request_id),
            })
        }
    }
}
```

**Error Response**:
```json
{
  "error": "InternalServerError",
  "message": "Request ID: 550e8400-e29b-41d4-a716-446655440000",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**User Support**:
- User provides request ID
- Support team searches logs
- Finds exact request trace

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_generates_request_id() {
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(middleware::from_fn(request_id_middleware));
    
    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    // Should have request ID header
    assert!(response.headers().get("x-request-id").is_some());
    
    // Should be valid UUID
    let request_id = response.headers().get("x-request-id").unwrap();
    assert!(Uuid::parse_str(request_id.to_str().unwrap()).is_ok());
}

#[tokio::test]
async fn test_preserves_existing_request_id() {
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(middleware::from_fn(request_id_middleware));
    
    let existing_id = Uuid::new_v4();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("X-Request-ID", existing_id.to_string())
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Should preserve existing ID
    let returned_id = response.headers().get("x-request-id").unwrap();
    assert_eq!(returned_id.to_str().unwrap(), existing_id.to_string());
}
```

---

## Production Patterns

### Pattern 1: Request ID in All Logs

```rust
// Configure tracing to include request ID in all spans
use tracing_subscriber::layer::SubscriberExt;

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true))
    .init();
```

### Pattern 2: Request ID in Error Responses

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = /* extract from context */;
        
        let body = json!({
            "error": self.error_type(),
            "message": self.message(),
            "request_id": request_id.to_string(),
        });
        
        (self.status_code(), Json(body)).into_response()
    }
}
```

### Pattern 3: Metrics Tagging

```rust
use metrics::{counter, histogram};

pub async fn handler(Extension(request_id): Extension<Uuid>) {
    counter!("requests_total", "endpoint" => "login").increment(1);
    
    let start = Instant::now();
    // ... process request ...
    let duration = start.elapsed();
    
    histogram!("request_duration_seconds", "endpoint" => "login")
        .record(duration.as_secs_f64());
}
```

---

## Observability Integration

### OpenTelemetry

```rust
use opentelemetry::trace::{Tracer, SpanKind};

pub async fn request_id_with_otel(
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = /* ... generate/extract ... */;
    
    let tracer = opentelemetry::global::tracer("auth-api");
    let span = tracer
        .span_builder("http_request")
        .with_kind(SpanKind::Server)
        .with_attributes(vec![
            KeyValue::new("request_id", request_id.to_string()),
        ])
        .start(&tracer);
    
    let _guard = span.enter();
    
    req.extensions_mut().insert(request_id);
    next.run(req).await
}
```

### Datadog APM

```rust
use datadog_trace::{Span, Tracer};

pub async fn request_id_with_datadog(
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = /* ... */;
    
    let mut span = Span::new("http.request");
    span.set_tag("request_id", request_id.to_string());
    
    req.extensions_mut().insert(request_id);
    let response = next.run(req).await;
    
    span.finish();
    response
}
```

---

## Best Practices

### 1. Always Include in Logs

```rust
// Good
info!(request_id = %request_id, "Processing request");

// Bad
info!("Processing request");  // No correlation
```

### 2. Return in Error Responses

```rust
// Good
{
  "error": "ValidationError",
  "message": "Invalid email",
  "request_id": "550e8400-..."
}

// Bad
{
  "error": "ValidationError",
  "message": "Invalid email"
}
```

### 3. Propagate to External Services

```rust
// Good
client.get(url)
    .header("X-Request-ID", request_id.to_string())
    .send()
    .await

// Bad
client.get(url).send().await  // Lost tracing
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `uuid` | Request ID generation |

---

## Related Files

- [middleware/security_headers.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/middleware/security_headers.md) - Security headers
- [middleware/rate_limit.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/middleware/rate_limit.md) - Rate limiting
- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 36  
**Security Level**: MEDIUM
