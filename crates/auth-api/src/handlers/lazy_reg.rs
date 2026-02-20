//! Lazy Registration Handler
//!
//! Exposes Just-in-Time (JIT) account creation functionality.

use crate::error::ApiError;
use auth_core::models::user::IdentifierType;
use auth_core::services::lazy_registration::LazyRegistrationService;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LazyRegisterRequest {
    pub tenant_id: Uuid,
    pub identifier: String,
    pub identifier_type: String, // "email" or "phone"
}

#[derive(Debug, Serialize)]
pub struct LazyRegisterResponse {
    pub user_id: Uuid,
    pub is_new: bool,
    pub status: String,
}

/// POST /auth/register/lazy
pub async fn lazy_register(
    State(lazy_reg_service): State<Arc<LazyRegistrationService>>,
    Json(payload): Json<LazyRegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let identifier_type = match payload.identifier_type.as_str() {
        "email" => IdentifierType::Email,
        "phone" => IdentifierType::Phone,
        _ => {
            return Err(ApiError::new(
                auth_core::error::AuthError::ValidationError {
                    message: "Invalid identifier type".to_string(),
                },
            ))
        }
    };

    let (user, is_new) = lazy_reg_service
        .get_or_create_user(payload.tenant_id, &payload.identifier, identifier_type)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::OK,
        Json(LazyRegisterResponse {
            user_id: user.id,
            is_new,
            status: user.status.to_string(),
        }),
    ))
}
