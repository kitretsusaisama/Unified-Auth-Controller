use crate::error::AuthError;
use crate::models::{Session, User};
use crate::services::risk_assessment::{RiskAssessor, RiskContext};
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: Session) -> Result<Session, AuthError>;
    async fn get(&self, session_token: &str) -> Result<Option<Session>, AuthError>;
    async fn delete(&self, session_token: &str) -> Result<(), AuthError>;
    async fn delete_by_user(&self, user_id: Uuid) -> Result<(), AuthError>;
}

pub struct SessionService {
    store: Arc<dyn SessionStore>,
    risk_engine: Arc<dyn RiskAssessor>,
}

impl SessionService {
    pub fn new(store: Arc<dyn SessionStore>, risk_engine: Arc<dyn RiskAssessor>) -> Self {
        Self { store, risk_engine }
    }

    pub async fn create_session(&self, user: User, risk_context: RiskContext) -> Result<Session, AuthError> {
        // 1. Assess Risk
        let risk_assessment = self.risk_engine.assess_risk(risk_context.clone()).await?;

        // 2. Decide Policy (e.g., if Critical, reject login)
        // using string matching for simplicity since RiskLevel is enum
        // Logic: if risk > 0.8 (Critical), assume we might want to challenge or block.
        // For this implementation, we log/record but proceed unless strictly blocked.
        // But let's block critical.
        if risk_assessment.score >= 0.9 {
             return Err(AuthError::AuthorizationDenied {
                 permission: "login".to_string(),
                 resource: "session".to_string()
             });
        }

        // 3. Create Session
        let session = Session {
            id: Uuid::new_v4(),
            user_id: user.id,
            tenant_id: Uuid::new_v4(), // Should come from context/user_tenant
            session_token: Uuid::new_v4().to_string(), // In real app, use high entropy random string
            device_fingerprint: risk_context.device_fingerprint,
            user_agent: risk_context.user_agent,
            ip_address: risk_context.ip_address,
            risk_score: risk_assessment.score,
            last_activity: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(60), // Configurable
            created_at: Utc::now(),
        };

        self.store.create(session).await
    }

    pub async fn validate_session(&self, token: &str) -> Result<Session, AuthError> {
        let session = self.store.get(token).await?
            .ok_or(AuthError::AuthenticationFailed { reason: "Session invalid".to_string() })?;

        if session.expires_at < Utc::now() {
            self.store.delete(token).await?;
            return Err(AuthError::AuthenticationFailed { reason: "Session expired".to_string() });
        }

        Ok(session)
    }

    pub async fn revoke_session(&self, token: &str) -> Result<(), AuthError> {
        self.store.delete(token).await
    }

    pub async fn revoke_user_sessions(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.store.delete_by_user(user_id).await
    }
}
