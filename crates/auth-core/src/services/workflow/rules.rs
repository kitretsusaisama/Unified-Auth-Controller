use super::FlowContext;
use crate::error::AuthError;

pub trait Rule: Send + Sync {
    fn evaluate(&self, ctx: &FlowContext) -> Result<bool, AuthError>;
}

#[derive(Default)]
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
                return Err(AuthError::AuthorizationDenied {
                    permission: "workflow".to_string(),
                    resource: "rule_violation".to_string(),
                });
            }
        }
        Ok(())
    }
}

pub struct MaxAttemptsRule {
    pub max: u32,
}

impl Rule for MaxAttemptsRule {
    fn evaluate(&self, ctx: &FlowContext) -> Result<bool, AuthError> {
        if let Some(attempts) = ctx.data.get("attempts").and_then(|v| v.as_u64()) {
            if attempts >= self.max as u64 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub struct RiskRule {
    pub threshold: f32,
}

impl Rule for RiskRule {
    fn evaluate(&self, ctx: &FlowContext) -> Result<bool, AuthError> {
        // Evaluate risk score from context
        if let Some(score) = ctx.data.get("risk_score").and_then(|v| v.as_f64()) {
            // If score > threshold, we might return false to block,
            // or true if this rule is "check if safe".
            // Let's assume this rule checks "Is Acceptable".
            // High score = Bad.
            if score as f32 > self.threshold {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
