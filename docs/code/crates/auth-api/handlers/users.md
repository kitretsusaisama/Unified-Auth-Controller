# handlers/users.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/users.rs`  
**Crate**: `auth-api`  
**Module**: `handlers::users`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - User management operations

## Purpose

Provides HTTP handlers for user management operations including account suspension and activation (admin-only operations).

### Problem It Solves

- User account lifecycle management
- Admin-controlled user suspension
- Account activation/reactivation
- User moderation capabilities

---

## Detailed Code Breakdown

### Function: `ban_user()`

**Signature**:
```rust
pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError>
```

**Purpose**: Suspend user account (admin only)

**OpenAPI Documentation**:
```rust
#[utoipa::path(
    post,
    path = "/users/{id}/ban",
    params(
        ("id" = Uuid, Path, description = "User ID to ban")
    ),
    responses(
        (status = 200, description = "User suspended successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "User Management"
)]
```

**Process**:
1. Extract user ID from path
2. Call identity service to suspend user
3. Return success response

**Response**:
```json
{
  "status": "success",
  "message": "User suspended"
}
```

**Example**:
```bash
curl -X POST https://api.example.com/users/550e8400-e29b-41d4-a716-446655440000/ban \
  -H "Authorization: Bearer <admin_token>"
```

---

### Function: `activate_user()`

**Signature**:
```rust
pub async fn activate_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError>
```

**Purpose**: Activate suspended user account (admin only)

**OpenAPI Documentation**:
```rust
#[utoipa::path(
    post,
    path = "/users/{id}/activate",
    params(
        ("id" = Uuid, Path, description = "User ID to activate")
    ),
    responses(
        (status = 200, description = "User activated successfully"),
        (status = 404, description = "User not found")
    ),
    tag = "User Management"
)]
```

**Process**:
1. Extract user ID from path
2. Call identity service to activate user
3. Return success response

**Response**:
```json
{
  "status": "success",
  "message": "User activated"
}
```

**Example**:
```bash
curl -X POST https://api.example.com/users/550e8400-e29b-41d4-a716-446655440000/activate \
  -H "Authorization: Bearer <admin_token>"
```

---

## Security Considerations

### 1. Admin-Only Access

**Requirement**: These endpoints must be protected by admin authorization

**Middleware**:
```rust
Router::new()
    .route("/users/:id/ban", post(ban_user))
    .route("/users/:id/activate", post(activate_user))
    .layer(require_role("admin"))
```

### 2. Audit Logging

**Implementation**:
```rust
pub async fn ban_user(
    State(state): State<AppState>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    // Ban user
    state.identity_service.ban_user(user_id).await?;
    
    // Audit log
    audit_log(AuditEvent {
        actor: admin_id,
        action: "ban_user".to_string(),
        target: user_id,
        timestamp: Utc::now(),
    }).await;
    
    Ok(Json(json!({"status": "success", "message": "User suspended"})))
}
```

### 3. Prevent Self-Ban

**Validation**:
```rust
pub async fn ban_user(
    State(state): State<AppState>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    // Prevent admin from banning themselves
    if admin_id == user_id {
        return Err(ApiError::new(AuthError::ValidationError {
            message: "Cannot ban yourself".to_string(),
        }));
    }
    
    state.identity_service.ban_user(user_id).await?;
    Ok(Json(json!({"status": "success"})))
}
```

---

## Extended User Management

### Additional Endpoints

```rust
/// List all users (admin only)
#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of users")
    ),
    tag = "User Management"
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Vec<User>>, ApiError> {
    let users = state.user_repo.list_users(params).await?;
    Ok(Json(users))
}

/// Get user details (admin only)
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details"),
        (status = 404, description = "User not found")
    ),
    tag = "User Management"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let user = state.user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound { user_id })?;
    Ok(Json(user))
}

/// Update user (admin only)
#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated"),
        (status = 404, description = "User not found")
    ),
    tag = "User Management"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<User>, ApiError> {
    let user = state.user_repo.update(user_id, payload).await?;
    Ok(Json(user))
}

/// Delete user (admin only)
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found")
    ),
    tag = "User Management"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.user_repo.delete(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

---

## Router Integration

```rust
use axum::Router;
use crate::handlers::users::*;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/:id/ban", post(ban_user))
        .route("/users/:id/activate", post(activate_user))
        .layer(require_role("admin")) // Admin-only
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_ban_user() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users/550e8400-e29b-41d4-a716-446655440000/ban")
                .header("Authorization", "Bearer admin_token")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activate_user() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/users/550e8400-e29b-41d4-a716-446655440000/activate")
                .header("Authorization", "Bearer admin_token")
                .body(Body::empty())
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
| `uuid` | User identifiers |
| `serde_json` | JSON responses |
| `utoipa` | OpenAPI documentation |

### Internal Dependencies

- [services/identity.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/identity.md) - Identity service

---

## Related Files

- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Auth handlers
- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 48  
**Security Level**: CRITICAL
