use axum::{extract::{State, Path}, Json};
use crate::AppState;
use crate::error::ApiError;
use uuid::Uuid;
use serde_json::json;

/// Suspend a user account (Admin only)
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
pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.identity_service.ban_user(user_id).await?;
    Ok(Json(json!({"status": "success", "message": "User suspended"})))
}

/// Activate a suspended user account (Admin only)
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
pub async fn activate_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.identity_service.activate_user(user_id).await?;
    Ok(Json(json!({"status": "success", "message": "User activated"})))
}
