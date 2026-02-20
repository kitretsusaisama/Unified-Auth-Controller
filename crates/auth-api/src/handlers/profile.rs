//! Profile Completion Handler with Real Logic
//!

use crate::error::ApiError;
use auth_core::error::{AuthError, TokenErrorKind};
use auth_core::models::{Claims, User};
use auth_core::services::identity::IdentityService;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CompleteProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub profile: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CompleteProfileResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Uuid,
    pub user: User,
}

pub async fn complete_profile(
    State(identity_service): State<Arc<IdentityService>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CompleteProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        ApiError::new(AuthError::TokenError {
            kind: TokenErrorKind::Invalid,
        })
    })?;

    // 1. Update Password if provided
    if let Some(password) = payload.password {
        if password.len() < 8 {
            return Err(ApiError::new(AuthError::ValidationError {
                message: "Password must be at least 8 chars".to_string(),
            }));
        }
        identity_service
            .update_password(user_id, password)
            .await
            .map_err(|_| ApiError::new(AuthError::InternalError))?;
    }

    // 2. Fetch current user to merge profile
    let mut user = identity_service
        .get_user(user_id)
        .await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    // 3. Merge Profile Data
    // Simple top-level merge for now. Deep merge would be better.
    if let serde_json::Value::Object(mut current_map) = user.profile_data {
        if let serde_json::Value::Object(new_map) = payload.profile {
            for (k, v) in new_map {
                current_map.insert(k, v);
            }
        }
        // Mark as complete
        current_map.insert("is_profile_complete".to_string(), json!(true));

        let merged_profile = serde_json::Value::Object(current_map);

        // 4. Update User
        user = identity_service
            .update_profile(user_id, merged_profile)
            .await
            .map_err(|_| ApiError::new(AuthError::InternalError))?;
    }

    Ok((
        StatusCode::OK,
        Json(CompleteProfileResponse {
            success: true,
            message: "Profile completed successfully".to_string(),
            user_id,
            user,
        }),
    ))
}
