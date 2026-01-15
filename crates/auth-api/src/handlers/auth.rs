use axum::{Json, extract::{State, Extension}};
use crate::AppState;
use auth_core::services::identity::{AuthRequest, AuthResponse};
use auth_core::models::user::{CreateUserRequest, User};
use crate::error::ApiError;
use crate::validation;
use uuid::Uuid;
use tracing::{info, warn};

/// Authenticate user and issue tokens
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AuthRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 423, description = "Account locked"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "Authentication"
)]
pub async fn login(
    State(state): State<AppState>,
    Extension(request_id): Extension<Uuid>,
    Json(mut payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Normalize email
    payload.email = validation::validate_email(&payload.email)
        .map_err(|e| ApiError::new(e).with_request_id(request_id))?;

    info!(
        request_id = %request_id,
        email = %payload.email,
        "Login attempt"
    );

    match state.identity_service.login(payload.clone()).await {
        Ok(response) => {
            info!(
                request_id = %request_id,
                email = %payload.email,
                "Login successful"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(
                request_id = %request_id,
                email = %payload.email,
                error = ?e,
                "Login failed"
            );
            Err(ApiError::new(e).with_request_id(request_id))
        }
    }
}

/// Register a new user account
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Registration successful", body = User),
        (status = 409, description = "Email already exists"),
        (status = 400, description = "Validation error"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "Authentication"
)]
pub async fn register(
    State(state): State<AppState>,
    Extension(request_id): Extension<Uuid>,
    Json(mut payload): Json<CreateUserRequest>,
) -> Result<Json<User>, ApiError> {
    // Validate and normalize email
    payload.email = Some(validation::validate_email(payload.email.as_deref().unwrap_or(""))
        .map_err(|e| ApiError::new(e).with_request_id(request_id))?);

    // Validate password strength
    if let Some(ref password) = payload.password {
        validation::validate_password(password)
            .map_err(|e| ApiError::new(e).with_request_id(request_id))?;
    } else {
        return Err(ApiError::new(auth_core::error::AuthError::ValidationError {
            message: "Password is required".to_string(),
        }).with_request_id(request_id));
    }

    info!(
        request_id = %request_id,
        email = %payload.email.as_deref().unwrap_or("<no email>"),
        "Registration attempt"
    );

    // TODO: Extract tenant_id from Host header or specialized middleware
    // For now, we default to the "default" tenant or nil
    let tenant_id = Uuid::default(); 

    match state.identity_service.register(payload.clone(), tenant_id).await {
        Ok(user) => {
            info!(
                request_id = %request_id,
                email = %payload.email.as_deref().unwrap_or("<no email>"),
                user_id = %user.id,
                "Registration successful"
            );
            Ok(Json(user))
        }
        Err(e) => {
            warn!(
                request_id = %request_id,
                email = %payload.email.as_deref().unwrap_or("<no email>"),
                error = ?e,
                "Registration failed"
            );
            Err(ApiError::new(e).with_request_id(request_id))
        }
    }
}
