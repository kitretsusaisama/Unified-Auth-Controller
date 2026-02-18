//! Lazy User Upgrade Workflow
//! Steps: Init -> SetCredentials -> EnrichProfile -> Success

use async_trait::async_trait;
use auth_core::services::workflow::{FlowContext, FlowAction, FlowState, StepHandler};
use auth_core::error::AuthError;

pub struct SetCredentialsStep;

#[async_trait]
impl StepHandler for SetCredentialsStep {
    async fn handle(&self, _ctx: &mut FlowContext, action: FlowAction) -> Result<FlowState, AuthError> {
        if action.name != "set_password" {
             return Err(AuthError::ValidationError { message: "Expected set_password action".to_string() });
        }

        let _password = action.payload.get("password")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::ValidationError { message: "Password required".to_string() })?;

        // Validate password complexity...
        // Update user (Upgrade lazy to full)
        // state.identity_service.update_password(ctx.user_id, password)

        Ok(FlowState::ProfileRequired)
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> {
        Ok(())
    }
}
