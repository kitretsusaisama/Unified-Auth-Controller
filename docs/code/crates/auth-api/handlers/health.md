# handlers/health.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/health.rs`  
**Crate**: `auth-api`  
**Module**: `handlers::health`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ❌ **NO** - Public health check endpoint

## Purpose

Provides simple health check endpoint for monitoring, load balancers, and orchestration systems to verify service availability.

### Problem It Solves

- Service health monitoring
- Load balancer health checks
- Kubernetes liveness/readiness probes
- Uptime monitoring
- Service discovery

---

## Detailed Code Breakdown

### Function: `health_check()`

**Signature**: `pub async fn health_check() -> impl IntoResponse`

**Purpose**: Return service health status

**OpenAPI Documentation**:
```rust
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy")
    ),
    tag = "Health"
)]
```

**Response**:
```json
{
  "status": "ok",
  "message": "SSO Platform API is healthy",
  "version": "0.1.0"
}
```

---

## Implementation

```rust
pub async fn health_check() -> impl IntoResponse {
    const MESSAGE: &str = "SSO Platform API is healthy";
    Json(json!({
        "status": "ok",
        "message": MESSAGE,
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

**Fields**:
- `status`: Always "ok" for successful response
- `message`: Human-readable status message
- `version`: Package version from `Cargo.toml`

---

## Usage Patterns

### Pattern 1: Load Balancer Health Check

**AWS ALB**:
```yaml
HealthCheck:
  Path: /health
  Protocol: HTTP
  Port: 8080
  HealthyThresholdCount: 2
  UnhealthyThresholdCount: 3
  TimeoutSeconds: 5
  IntervalSeconds: 30
```

**Response**:
- **200 OK**: Service healthy, route traffic
- **Non-200**: Service unhealthy, stop routing

---

### Pattern 2: Kubernetes Probes

**Liveness Probe**:
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3
```

**Readiness Probe**:
```yaml
readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 2
```

---

### Pattern 3: Monitoring

**Prometheus**:
```yaml
scrape_configs:
  - job_name: 'sso-api'
    metrics_path: '/metrics'
    static_configs:
      - targets: ['localhost:8080']
    
    # Health check for service discovery
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
```

**Uptime Monitoring**:
```bash
# Pingdom, UptimeRobot, etc.
curl -f https://api.example.com/health || alert_team
```

---

## Advanced Health Checks

### Enhanced Health Endpoint

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    pub database: String,
    pub redis: String,
    pub external_api: String,
}

pub async fn health_check_detailed(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - state.start_time;
    
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        checks: HealthChecks {
            database: check_database(&state.pool).await,
            redis: check_redis(&state.redis).await,
            external_api: check_external_api().await,
        },
    })
}

async fn check_database(pool: &MySqlPool) -> String {
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => "healthy".to_string(),
        Err(_) => "unhealthy".to_string(),
    }
}
```

**Response**:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime": 3600,
  "checks": {
    "database": "healthy",
    "redis": "healthy",
    "external_api": "healthy"
  }
}
```

---

### Readiness vs Liveness

**Liveness** (`/health/live`):
- Service process is running
- No deadlocks
- Basic functionality works

```rust
pub async fn liveness() -> impl IntoResponse {
    // Simple check - process is alive
    Json(json!({"status": "alive"}))
}
```

**Readiness** (`/health/ready`):
- All dependencies available
- Database connected
- Can serve traffic

```rust
pub async fn readiness(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check database
    if !check_database(&state.pool).await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    // Check Redis
    if !check_redis(&state.redis).await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    Ok(Json(json!({"status": "ready"})))
}
```

---

## Router Integration

```rust
use axum::Router;
use crate::handlers::health::health_check;

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_health_check() {
    let response = health_check().await.into_response();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = extract_json_body(response).await;
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Production Considerations

### 1. No Authentication

Health checks should be **unauthenticated** for load balancers

```rust
// Don't require auth for health checks
Router::new()
    .route("/health", get(health_check)) // Public
    .route("/api/*", /* ... */)          // Authenticated
    .layer(auth_middleware)
```

### 2. Minimal Dependencies

Keep health check **fast and simple**

```rust
// Good: Fast, no dependencies
pub async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

// Bad: Slow, many dependencies
pub async fn health_check() -> impl IntoResponse {
    check_database().await;
    check_redis().await;
    check_external_api().await;
    // Takes 5+ seconds...
}
```

### 3. Separate Endpoints

Use different endpoints for different purposes

- `/health` - Simple liveness
- `/health/ready` - Readiness with dependencies
- `/health/detailed` - Full diagnostic (authenticated)

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `serde_json` | JSON responses |
| `utoipa` | OpenAPI documentation |

---

## Related Files

- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup
- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Auth handlers

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 21  
**Security Level**: LOW
