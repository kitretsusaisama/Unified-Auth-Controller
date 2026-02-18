use axum::{
    extract::{State, Path, Json},
    response::IntoResponse,
    http::StatusCode,
};
use uuid::Uuid;
use crate::AppState;
use crate::error::ApiError;
use auth_core::models::CreateRoleRequest;

// ============================================================================
// Create Role
// ============================================================================

pub async fn create_role(
    State(state): State<AppState>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Authenticate & Authorize (Middleware or Manual Check)
    // For now, assuming caller has permission (e.g. from token in middleware)

    // 2. Validate Request
    // 3. Call Service
    let tenant_id = Uuid::new_v4(); // Mock tenant for now, should come from auth context

    // 4. Create Role via AuthorizationService (not generic RoleService)
    // We need to inject AuthorizationService into AppState or use RoleService if updated.
    // The previous step updated RoleService to handle the new Role struct.

    let role = state.role_service.create_role(tenant_id, payload).await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(role)))
}

// ============================================================================
// Get Role
// ============================================================================

pub async fn get_role(
    State(_state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    // Stub
    Ok(Json(serde_json::json!({"id": role_id, "name": "Stub Role"})))
}
