//! Audit Logging System
//!
//! Structured logging for security-critical events.
//! Compliant with MNC audit requirements.

use chrono::{DateTime, Utc};
use serde::{Serialize, Serializer};
use uuid::Uuid;

/// Categories of audit events
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCategory {
    Authentication,
    Authorization,
    UserManagement,
    System,
    Security,
}

/// Severity levels for audit events
#[derive(Debug, Clone)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
}

impl Serialize for AuditSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            AuditSeverity::Info => "INFO",
            AuditSeverity::Warning => "WARNING",
            AuditSeverity::Critical => "CRITICAL",
        };
        serializer.serialize_str(s)
    }
}

/// Structured Audit Event
#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: AuditCategory,
    pub action: String,
    pub severity: AuditSeverity,
    pub actor_id: Option<Uuid>,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub outcome: AuditOutcome,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure { reason: String },
}

impl AuditEvent {
    pub fn new(
        category: AuditCategory,
        action: impl Into<String>,
        severity: AuditSeverity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            action: action.into(),
            severity,
            actor_id: None,
            resource_id: None,
            ip_address: None,
            user_agent: None,
            tenant_id: None,
            metadata: serde_json::json!({}),
            outcome: AuditOutcome::Success,
        }
    }

    pub fn with_actor(mut self, actor_id: Uuid) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    pub fn with_resource(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    pub fn with_context(
        mut self,
        ip: Option<String>,
        ua: Option<String>,
        tenant: Option<Uuid>,
    ) -> Self {
        self.ip_address = ip;
        self.user_agent = ua;
        self.tenant_id = tenant;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn failure(mut self, reason: impl Into<String>) -> Self {
        self.outcome = AuditOutcome::Failure {
            reason: reason.into(),
        };
        self
    }
}

/// Trait for recording audit events
#[async_trait::async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, event: AuditEvent);
}

/// Implementation using `tracing` for structured output (can be piped to ELK/Splunk)
pub struct TracingAuditLogger;

#[async_trait::async_trait]
impl AuditLogger for TracingAuditLogger {
    async fn log(&self, event: AuditEvent) {
        // We use a specific target "audit" so these logs can be filtered/routed separately
        tracing::info!(
            target: "audit",
            event_id = %event.id,
            timestamp = %event.timestamp,
            category = ?event.category,
            action = %event.action,
            severity = ?event.severity,
            actor_id = ?event.actor_id,
            outcome = ?event.outcome,
            payload = ?serde_json::to_string(&event).unwrap_or_default(),
            "AUDIT_EVENT"
        );
    }
}
