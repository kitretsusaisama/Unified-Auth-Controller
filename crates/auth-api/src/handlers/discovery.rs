use axum::{
    extract::Json,
    response::IntoResponse,
    http::StatusCode,
};
use auth_protocols::discovery::generate_oidc_metadata;
use std::env;

/// GET /.well-known/openid-configuration
pub async fn oidc_configuration() -> Result<impl IntoResponse, StatusCode> {
    let base_url = env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let metadata = generate_oidc_metadata(&base_url);

    Ok(Json(metadata))
}
