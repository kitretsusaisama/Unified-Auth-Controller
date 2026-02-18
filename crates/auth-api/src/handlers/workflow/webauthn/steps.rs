use async_trait::async_trait;
use auth_core::services::workflow::{FlowContext, FlowAction, FlowState, StepHandler};
use auth_core::error::AuthError;

// Placeholder for WebAuthn integration
// In a real implementation, we would inject a WebAuthnService (wrapping webauthn-rs)
// into the handler context or via state injection.

pub struct WebAuthnStartStep;

#[async_trait]
impl StepHandler for WebAuthnStartStep {
    async fn handle(&self, ctx: &mut FlowContext, action: FlowAction) -> Result<FlowState, AuthError> {
        if action.name != "start_webauthn" {
             return Err(AuthError::ValidationError { message: "Invalid action".to_string() });
        }

        // 1. Generate Challenge via WebAuthnService
        // let (challenge, state) = webauthn_service.start_authentication(user).await?;

        // 2. Store challenge in context
        // ctx.data.insert("webauthn_challenge", ...);

        // 3. Return response with challenge data for UI
        // Ideally FlowResult supports 'data' payload, but we put it in context and
        // the engine extracts UI hints. We might need to extend FlowResult logic.

        // For simulation:
        let challenge = "mock_challenge_base64";
        ctx.data.insert("webauthn_challenge".to_string(), serde_json::Value::String(challenge.to_string()));

        Ok(FlowState::Custom("WebAuthnVerify".to_string()))
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> { Ok(()) }
}

pub struct WebAuthnVerifyStep;

#[async_trait]
impl StepHandler for WebAuthnVerifyStep {
    async fn handle(&self, _ctx: &mut FlowContext, action: FlowAction) -> Result<FlowState, AuthError> {
        if action.name != "submit_webauthn" {
             return Err(AuthError::ValidationError { message: "Invalid action".to_string() });
        }

        let _credential = action.payload.get("credential")
            .ok_or(AuthError::ValidationError { message: "Credential required".to_string() })?;

        // 1. Retrieve challenge from context
        // 2. Verify with WebAuthnService

        // Simulation:
        Ok(FlowState::Success)
    }

    async fn validate(&self, _ctx: &FlowContext) -> Result<(), AuthError> { Ok(()) }
}
