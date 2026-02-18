//! Magic Link Workflow Step
//! Allows users to resume a flow by clicking a link (simulated via token verification)

use async_trait::async_trait;
use auth_core::services::workflow::{FlowContext, FlowAction, FlowState, StepHandler};
use auth_core::error::AuthError;

pub struct MagicLinkStep;

#[async_trait]
impl StepHandler for MagicLinkStep {
    async fn handle(&self, _ctx: &mut FlowContext, action: FlowAction) -> Result<FlowState, AuthError> {
        if action.name != "verify_magic_link" {
             return Err(AuthError::ValidationError { message: "Invalid action".to_string() });
        }

        let token = action.payload.get("token")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::ValidationError { message: "Token required".to_string() })?;

        // 1. Verify token (usually against a separate MagicLink table or signed JWT)
        // 2. Check expiry
        // 3. Mark context as authenticated

        // Simulation:
        if token == "valid_magic_token" {
             // If we had the user ID in the token, we'd set it here.
             // ctx.user_id = Some(...);
             Ok(FlowState::Success)
        } else {
             Err(AuthError::InvalidCredentials)
        }
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> { Ok(()) }
}
