use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::error::AuthError;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FlowState {
    Start,
    Identify,
    Authenticate,
    MfaRequired,
    ConsentRequired,
    ProfileRequired,
    VerifyIdentifier, // For registration
    SetCredentials,   // For registration
    Success,
    Failed,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowContext {
    pub flow_id: String,
    pub tenant_id: Uuid,
    pub flow_type: String, // "login", "register", "lazy_upgrade"
    pub current_state: FlowState,
    pub user_id: Option<Uuid>,
    pub data: HashMap<String, serde_json::Value>,
    pub version: u64, // Optimistic locking
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct FlowAction {
    pub name: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct FlowResult {
    pub next_state: FlowState,
    pub ui_hints: Option<HashMap<String, serde_json::Value>>,
    pub error: Option<String>,
}

#[async_trait]
pub trait StepHandler: Send + Sync {
    async fn handle(&self, ctx: &mut FlowContext, action: FlowAction) -> Result<FlowState, AuthError>;
    async fn validate(&self, ctx: &FlowContext) -> Result<(), AuthError>;
}

pub struct WorkflowEngine {
    handlers: HashMap<FlowState, Box<dyn StepHandler>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, state: FlowState, handler: Box<dyn StepHandler>) {
        self.handlers.insert(state, handler);
    }

    pub async fn process(&self, mut ctx: FlowContext, action: FlowAction) -> Result<(FlowContext, FlowResult), AuthError> {
        // 1. Validate version (Optimistic Lock check would happen at persistence layer, but we can check logic here)

        // 2. Get Handler for current state
        let handler = self.handlers.get(&ctx.current_state)
            .ok_or(AuthError::InternalError)?; // "No handler for state"

        // 3. Execute Handler
        let next_state = handler.handle(&mut ctx, action).await?;

        // 4. Update Context
        ctx.current_state = next_state.clone();
        ctx.updated_at = chrono::Utc::now().timestamp();
        ctx.version += 1;

        // 5. Determine UI Hints based on new state
        let hints = self.get_ui_hints(&next_state);

        Ok((ctx, FlowResult {
            next_state,
            ui_hints: Some(hints),
            error: None,
        }))
    }

    fn get_ui_hints(&self, state: &FlowState) -> HashMap<String, serde_json::Value> {
        let mut hints = HashMap::new();
        match state {
            FlowState::Identify => {
                hints.insert("view".to_string(), "login_identifier".into());
                hints.insert("fields".to_string(), serde_json::json!(["email", "phone"]));
            },
            FlowState::Authenticate => {
                hints.insert("view".to_string(), "login_password".into());
            },
            FlowState::MfaRequired => {
                hints.insert("view".to_string(), "mfa_challenge".into());
            },
            _ => {}
        }
        hints
    }
}
