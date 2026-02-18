//! Advanced Authentication Flow Handler
//!
//! Implements a stateful, step-based authentication flow (MNC Grade).
//! Supports:
//! - Progressive Profiling (Placeholder)
//! - MFA Step-up
//! - Consent screens (Placeholder)
//! - Federated redirects (Placeholder)
//! - Recovery flows (Placeholder)

use axum::{
    extract::{Json, State, Path},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;
use crate::error::ApiError;
use auth_core::error::AuthError;
use std::time::Duration;
use chrono::Utc;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthFlowType {
    Login,
    Register,
    Recovery,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthFlowState {
    Identify,           // Waiting for username/email
    Authenticate,       // Waiting for password/credential
    MfaRequired,        // Waiting for MFA code
    ConsentRequired,    // Waiting for scope consent
    ProfileRequired,    // Progressive profiling
    Success,            // Flow complete, tokens issued
    Failed,             // Flow failed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthFlowContext {
    pub flow_id: String,
    pub tenant_id: Uuid,
    pub flow_type: AuthFlowType,
    pub current_state: AuthFlowState,
    pub user_id: Option<Uuid>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub attempts: u32,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct StartFlowRequest {
    pub flow_type: AuthFlowType,
    pub tenant_id: Uuid,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthFlowResponse {
    pub flow_id: String,
    pub state: AuthFlowState,
    pub next_step: Option<String>,
    pub available_factors: Option<Vec<String>>,
    pub error: Option<String>,
    // Success fields
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResumeFlowRequest {
    pub action: String, // "submit_identifier", "submit_password", "submit_mfa"
    pub data: serde_json::Value,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /auth/flow/start
/// Initiates a new stateful authentication flow
pub async fn start_flow(
    State(state): State<AppState>,
    Json(payload): Json<StartFlowRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Generate Flow ID (secure random)
    let flow_id = Uuid::new_v4().to_string();

    // 2. Initialize Context
    let context = AuthFlowContext {
        flow_id: flow_id.clone(),
        tenant_id: payload.tenant_id,
        flow_type: payload.flow_type,
        current_state: AuthFlowState::Identify,
        user_id: None,
        client_id: payload.client_id,
        redirect_uri: payload.redirect_uri,
        attempts: 0,
        created_at: Utc::now().timestamp(),
    };

    // 3. Store in Cache (TTL 15 mins)
    let key = format!("auth_flow:{}", flow_id);
    let val_str = serde_json::to_string(&context).map_err(|_| ApiError::new(AuthError::InternalError))?;

    state.cache.set(&key, &val_str, Duration::from_secs(900)).await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    Ok(Json(AuthFlowResponse {
        flow_id,
        state: AuthFlowState::Identify,
        next_step: Some("submit_identifier".to_string()),
        available_factors: None,
        error: None,
        access_token: None,
        refresh_token: None,
    }))
}

/// GET /auth/flow/:id
/// Retrieves current state of the flow
pub async fn get_flow_state(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let key = format!("auth_flow:{}", flow_id);

    let val_opt = state.cache.get(&key).await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    let val_str = val_opt.ok_or(ApiError::new(AuthError::ValidationError { message: "Flow expired or not found".to_string() }))?;
    let context: AuthFlowContext = serde_json::from_str(&val_str)
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    Ok(Json(AuthFlowResponse {
        flow_id,
        state: context.current_state,
        next_step: match context.current_state {
            AuthFlowState::Identify => Some("submit_identifier".to_string()),
            AuthFlowState::Authenticate => Some("submit_password".to_string()),
            AuthFlowState::MfaRequired => Some("submit_mfa".to_string()),
            AuthFlowState::Success => None,
            AuthFlowState::Failed => None,
            _ => None,
        },
        available_factors: None,
        error: None,
        access_token: None,
        refresh_token: None,
    }))
}

/// POST /auth/flow/:id/resume
/// Progresses the flow with user input
pub async fn resume_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(payload): Json<ResumeFlowRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let key = format!("auth_flow:{}", flow_id);
    let val_opt = state.cache.get(&key).await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    let val_str = val_opt.ok_or(ApiError::new(AuthError::ValidationError { message: "Flow expired or not found".to_string() }))?;
    let mut context: AuthFlowContext = serde_json::from_str(&val_str)
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    // Check attempts
    if context.attempts >= 5 {
        context.current_state = AuthFlowState::Failed;
        let val_str = serde_json::to_string(&context).unwrap_or_default();
        state.cache.set(&key, &val_str, Duration::from_secs(300)).await.ok();
        return Err(ApiError::new(AuthError::AuthorizationDenied { permission: "login".to_string(), resource: "flow".to_string() }));
    }

    match payload.action.as_str() {
        "submit_identifier" => {
            if context.current_state != AuthFlowState::Identify {
                 return Err(ApiError::new(AuthError::ValidationError { message: "Invalid state for this action".to_string() }));
            }

            let identifier = payload.data.get("identifier")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::new(AuthError::ValidationError { message: "identifier required".to_string() }))?;

            // Lookup user
            if let Ok(Some(user)) = state.identity_service.find_user_by_identifier(context.tenant_id, identifier).await {
                context.user_id = Some(user.id);
                context.current_state = AuthFlowState::Authenticate;
                // Save context
                let val_str = serde_json::to_string(&context).map_err(|_| ApiError::new(AuthError::InternalError))?;
                state.cache.set(&key, &val_str, Duration::from_secs(900)).await.map_err(|_| ApiError::new(AuthError::InternalError))?;

                Ok(Json(AuthFlowResponse {
                    flow_id,
                    state: AuthFlowState::Authenticate,
                    next_step: Some("submit_password".to_string()),
                    available_factors: Some(vec!["password".to_string()]),
                    error: None,
                    access_token: None,
                    refresh_token: None,
                }))
            } else {
                Err(ApiError::new(AuthError::UserNotFound))
            }
        },
        "submit_password" => {
            if context.current_state != AuthFlowState::Authenticate {
                 return Err(ApiError::new(AuthError::ValidationError { message: "Invalid state for this action".to_string() }));
            }

            let password = payload.data.get("password")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::new(AuthError::ValidationError { message: "password required".to_string() }))?;

            let user_id = context.user_id.ok_or(ApiError::new(AuthError::InternalError))?;

            // Verify Password
            match state.identity_service.verify_password(user_id, password).await {
                Ok(true) => {
                    // Password valid
                    // Check MFA
                    let user = state.identity_service.get_user(user_id).await.map_err(ApiError::from)?;
                    if user.mfa_enabled {
                        context.current_state = AuthFlowState::MfaRequired;
                        let val_str = serde_json::to_string(&context).map_err(|_| ApiError::new(AuthError::InternalError))?;
                        state.cache.set(&key, &val_str, Duration::from_secs(900)).await.map_err(|_| ApiError::new(AuthError::InternalError))?;

                        Ok(Json(AuthFlowResponse {
                            flow_id,
                            state: AuthFlowState::MfaRequired,
                            next_step: Some("submit_mfa".to_string()),
                            available_factors: Some(vec!["otp".to_string()]), // Simplified
                            error: None,
                            access_token: None,
                            refresh_token: None,
                        }))
                    } else {
                        // Success - Issue Tokens
                        let token_response = state.identity_service.issue_tokens_for_user(&user, context.tenant_id, context.client_id.clone(), None).await.map_err(ApiError::from)?;

                        context.current_state = AuthFlowState::Success;
                        // Clear flow or keep for audit? Delete to cleanup.
                        state.cache.delete(&key).await.ok();

                        Ok(Json(AuthFlowResponse {
                            flow_id,
                            state: AuthFlowState::Success,
                            next_step: None,
                            available_factors: None,
                            error: None,
                            access_token: Some(token_response.access_token),
                            refresh_token: Some(token_response.refresh_token),
                        }))
                    }
                },
                Ok(false) => {
                    context.attempts += 1;
                    let val_str = serde_json::to_string(&context).map_err(|_| ApiError::new(AuthError::InternalError))?;
                    state.cache.set(&key, &val_str, Duration::from_secs(900)).await.ok();
                    Err(ApiError::new(AuthError::InvalidCredentials))
                },
                Err(e) => Err(ApiError::from(e)),
            }
        },
        "submit_mfa" => {
             if context.current_state != AuthFlowState::MfaRequired {
                 return Err(ApiError::new(AuthError::ValidationError { message: "Invalid state for this action".to_string() }));
            }

            let code = payload.data.get("code")
                .and_then(|v| v.as_str())
                .ok_or(ApiError::new(AuthError::ValidationError { message: "code required".to_string() }))?;

            let user_id = context.user_id.ok_or(ApiError::new(AuthError::InternalError))?;

            let user = state.identity_service.get_user(user_id).await.map_err(ApiError::from)?;
            let secret = user.mfa_secret.clone().ok_or(ApiError::new(AuthError::ValidationError { message: "MFA not configured for user".to_string() }))?;

            // OtpService verify_totp takes (secret, code) - Synchronous
            let is_valid = state.otp_service.verify_totp(&secret, code).map_err(|_| ApiError::new(AuthError::InternalError))?;

            if is_valid {
                 // Success - Issue Tokens
                 let token_response = state.identity_service.issue_tokens_for_user(&user, context.tenant_id, context.client_id.clone(), None).await.map_err(ApiError::from)?;

                 context.current_state = AuthFlowState::Success;
                 state.cache.delete(&key).await.ok();

                 Ok(Json(AuthFlowResponse {
                     flow_id,
                     state: AuthFlowState::Success,
                     next_step: None,
                     available_factors: None,
                     error: None,
                     access_token: Some(token_response.access_token),
                     refresh_token: Some(token_response.refresh_token),
                 }))
            } else {
                 context.attempts += 1;
                 let val_str = serde_json::to_string(&context).map_err(|_| ApiError::new(AuthError::InternalError))?;
                 state.cache.set(&key, &val_str, Duration::from_secs(900)).await.ok();
                 Err(ApiError::new(AuthError::InvalidCredentials))
            }
        },
        _ => Err(ApiError::new(AuthError::ValidationError { message: "Invalid action".to_string() }))
    }
}
