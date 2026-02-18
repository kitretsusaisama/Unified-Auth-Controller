use crate::error::AuthError;
use super::FlowContext;

pub trait Rule: Send + Sync {
    fn evaluate(&self, ctx: &FlowContext) -> Result<bool, AuthError>;
}

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn check_all(&self, ctx: &FlowContext) -> Result<(), AuthError> {
        for rule in &self.rules {
            if !rule.evaluate(ctx)? {
                // Determine specific error? For now generic.
                return Err(AuthError::AuthorizationDenied {
                    permission: "workflow".to_string(),
                    resource: "rule_violation".to_string()
                });
            }
        }
        Ok(())
    }
}

// Example Rules
pub struct MaxAttemptsRule {
    pub max: u32,
}

impl Rule for MaxAttemptsRule {
    fn evaluate(&self, ctx: &FlowContext) -> Result<bool, AuthError> {
        // Assuming attempts are stored in data or a top-level field?
        // We added `attempts` to AuthFlowContext in handler, but FlowContext here has `data`.
        // Let's assume standard field or data mapping.
        // For generic engine, we check `data.get("attempts")`

        if let Some(attempts) = ctx.data.get("attempts").and_then(|v| v.as_u64()) {
            if attempts >= self.max as u64 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
