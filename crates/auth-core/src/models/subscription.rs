use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String, // e.g., "free", "pro", "enterprise"
    pub name: String,
    pub features: Vec<String>,
    pub quotas: HashMap<String, i64>, // e.g., "users" -> 5, "api_calls" -> 1000
    pub price_monthly: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TenantSubscription {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub current_usage: sqlx::types::Json<HashMap<String, i64>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Canceled,
    PastDue,
    Trialing,
}

// Manual implementation for sqlx::Type since it's an enum stored as string
impl sqlx::Type<sqlx::MySql> for SubscriptionStatus {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &sqlx::mysql::MySqlTypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::MySql> for SubscriptionStatus {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = <&str as sqlx::Decode<sqlx::MySql>>::decode(value)?;
        match s {
            "active" => Ok(SubscriptionStatus::Active),
            "canceled" => Ok(SubscriptionStatus::Canceled),
            "past_due" => Ok(SubscriptionStatus::PastDue),
            "trialing" => Ok(SubscriptionStatus::Trialing),
            _ => Err("Unknown subscription status".into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::MySql> for SubscriptionStatus {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> sqlx::encode::IsNull {
        let s = match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Canceled => "canceled",
            SubscriptionStatus::PastDue => "past_due",
            SubscriptionStatus::Trialing => "trialing",
        };
        <&str as sqlx::Encode<sqlx::MySql>>::encode_by_ref(&s, buf)
    }
}
