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
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    // Retrieve JWKS from the token engine (backed by KeyManager)
    let jwks = state.identity_service.get_jwks().await;
    Ok(Json(jwks))
}
