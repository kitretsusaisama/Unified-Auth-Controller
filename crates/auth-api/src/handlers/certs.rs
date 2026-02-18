use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use crate::AppState;

/// GET /auth/certs
/// Returns JWKS public keys
pub async fn jwks(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // Stub implementation returning 501 Not Implemented
    // In a future phase, this will retrieve keys from the TokenProvider
    Err::<Json<()>, StatusCode>(StatusCode::NOT_IMPLEMENTED)
}
