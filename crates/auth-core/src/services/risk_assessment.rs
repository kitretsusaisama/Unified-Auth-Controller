//! Risk assessment service for security evaluation

use crate::error::AuthError;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RiskContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub geolocation: Option<String>,
    pub previous_logins: Vec<LoginHistory>,
}

#[derive(Debug, Clone)]
pub struct LoginHistory {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ip_address: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub score: f32,
    pub level: RiskLevel,
    pub factors: Vec<RiskFactor>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f32,
    pub description: String,
}

#[async_trait::async_trait]
pub trait RiskAssessor: Send + Sync {
    async fn assess_risk(&self, context: RiskContext) -> Result<RiskAssessment, AuthError>;
    async fn update_user_risk_score(&self, user_id: Uuid, score: f32) -> Result<(), AuthError>;
}

use std::collections::HashSet;

#[derive(Default)]
pub struct RiskEngine {}

impl RiskEngine {
    pub fn new() -> Self {
        Self {}
    }

    fn calculate_ip_risk(
        &self,
        ip: &Option<String>,
        history: &[LoginHistory],
    ) -> (f32, Option<RiskFactor>) {
        if let Some(current_ip) = ip {
            let known_ips: HashSet<&String> = history.iter().map(|h| &h.ip_address).collect();
            if !known_ips.contains(current_ip) {
                return (
                    0.3,
                    Some(RiskFactor {
                        name: "new_ip".to_string(),
                        weight: 0.3,
                        description: "Login from new IP address".to_string(),
                    }),
                );
            }
        }
        (0.0, None)
    }

    // Placeholder for more complex checks (e.g., Geo-velocity)
}

#[async_trait::async_trait]
impl RiskAssessor for RiskEngine {
    async fn assess_risk(&self, context: RiskContext) -> Result<RiskAssessment, AuthError> {
        let mut score = 0.0;
        let mut factors = Vec::new();
        let mut recommendations = Vec::new();

        // 1. IP Risk
        let (ip_score, ip_factor) =
            self.calculate_ip_risk(&context.ip_address, &context.previous_logins);
        score += ip_score;
        if let Some(f) = ip_factor {
            factors.push(f);
        }

        // 2. Device Risk (Mock implementation - normally would check against user_devices table)
        if context.device_fingerprint.is_none() {
            score += 0.2;
            factors.push(RiskFactor {
                name: "missing_fingerprint".to_string(),
                weight: 0.2,
                description: "Device fingerprinting unavailable".to_string(),
            });
        }

        // 3. Recent Failures (Mock - assuming history contains failures)
        let recent_failures = context
            .previous_logins
            .iter()
            .filter(|l| !l.success)
            .count();
        if recent_failures > 3 {
            score += 0.4;
            factors.push(RiskFactor {
                name: "multiple_failures".to_string(),
                weight: 0.4,
                description: format!("{} recent failed attempts", recent_failures),
            });
            recommendations.push("Enable MFA".to_string());
        }

        let level = match score {
            s if s < 0.3 => RiskLevel::Low,
            s if s < 0.6 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(RiskAssessment {
            score,
            level,
            factors,
            recommendations,
        })
    }

    async fn update_user_risk_score(&self, _user_id: Uuid, _score: f32) -> Result<(), AuthError> {
        // In a real system, this would update the user's base risk profile in DB.
        // For now, it's a no-op or we can add a repo dependency later.
        Ok(())
    }
}
