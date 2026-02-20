use crate::error::ApiError;
use crate::AppState;
use async_trait::async_trait;
use auth_core::error::AuthError;
use auth_core::services::workflow::{
    FlowAction, FlowContext, FlowState, StepHandler, WorkflowEngine,
};
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};
use std::time::Duration;

// ============================================================================
// Step Handlers (Example implementations for Login Flow)
// ============================================================================

pub struct IdentifyStep;
#[async_trait]
impl StepHandler for IdentifyStep {
    async fn handle(
        &self,
        _ctx: &mut FlowContext,
        action: FlowAction,
    ) -> Result<FlowState, AuthError> {
        // Validate Action
        if action.name != "submit_identifier" {
            return Err(AuthError::ValidationError {
                message: "Invalid action".to_string(),
            });
        }

        let _identifier = action
            .payload
            .get("identifier")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::ValidationError {
                message: "Identifier required".to_string(),
            })?;

        // In real impl, we'd lookup user here.
        // ctx.user_id = Some(...);

        // For now, simulate success -> Authenticate
        Ok(FlowState::Authenticate)
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> {
        Ok(())
    }
}

pub struct AuthenticateStep;
#[async_trait]
impl StepHandler for AuthenticateStep {
    async fn handle(
        &self,
        _ctx: &mut FlowContext,
        action: FlowAction,
    ) -> Result<FlowState, AuthError> {
        if action.name != "submit_password" {
            return Err(AuthError::ValidationError {
                message: "Invalid action".to_string(),
            });
        }

        // Verify password logic...

        Ok(FlowState::Success)
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> {
        Ok(())
    }
}

// ============================================================================
// Universal Endpoint
// ============================================================================

#[derive(serde::Deserialize)]
pub struct SubmitFlowRequest {
    pub action: String,
    pub payload: serde_json::Value,
}

pub async fn submit(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(body): Json<SubmitFlowRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Load Context (with Lock/Version check logic implied by cache/DB)
    let key = format!("flow:{}", flow_id);
    let val_opt = state
        .cache
        .get(&key)
        .await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    let val_str = val_opt.ok_or(ApiError::new(AuthError::ValidationError {
        message: "Flow not found".to_string(),
    }))?;
    let context: FlowContext =
        serde_json::from_str(&val_str).map_err(|_| ApiError::new(AuthError::InternalError))?;

    // 2. Instantiate Engine & Register Handlers (Ideally this is done once or factory-based)
    let mut engine = WorkflowEngine::new();
    engine.register_handler(FlowState::Identify, Box::new(IdentifyStep));
    engine.register_handler(FlowState::Authenticate, Box::new(AuthenticateStep));

    // 3. Process
    let action = FlowAction {
        name: body.action,
        payload: body.payload,
    };
    let (new_ctx, result) = engine
        .process(context, action)
        .await
        .map_err(ApiError::from)?;

    // 4. Save Context (CAS/Version check should happen here)
    let new_val =
        serde_json::to_string(&new_ctx).map_err(|_| ApiError::new(AuthError::InternalError))?;
    state
        .cache
        .set(&key, &new_val, Duration::from_secs(900))
        .await
        .map_err(|_| ApiError::new(AuthError::InternalError))?;

    Ok(Json(result))
}
