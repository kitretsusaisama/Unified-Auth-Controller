use axum::{response::IntoResponse, Json};
use serde_json::json;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy")
    ),
    tag = "Health"
)]
pub async fn health_check() -> impl IntoResponse {
    const MESSAGE: &str = "SSO Platform API is healthy";
    Json(json!({
        "status": "ok",
        "message": MESSAGE,
        "version": env!("CARGO_PKG_VERSION")
    }))
}
