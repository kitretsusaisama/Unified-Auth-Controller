//! Session model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub session_token: String,
    pub device_fingerprint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub risk_score: f32,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}