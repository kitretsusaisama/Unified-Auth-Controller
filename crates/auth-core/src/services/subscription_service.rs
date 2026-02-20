use crate::error::AuthError;
use crate::models::subscription::{SubscriptionPlan, TenantSubscription, SubscriptionStatus};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;
use anyhow::Result;

// In a real app, this might be another repository or config file
const FREE_PLAN_ID: &str = "free";
const PRO_PLAN_ID: &str = "pro";

#[async_trait::async_trait]
pub trait SubscriptionStore: Send + Sync {
    async fn create(&self, sub: TenantSubscription) -> Result<TenantSubscription, AuthError>;
    async fn get_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantSubscription>, AuthError>;
    async fn update_usage(&self, tenant_id: Uuid, usage: HashMap<String, i64>) -> Result<(), AuthError>;
}

pub struct SubscriptionService {
    store: Arc<dyn SubscriptionStore>,
    plans: HashMap<String, SubscriptionPlan>,
}

impl SubscriptionService {
    pub fn new(store: Arc<dyn SubscriptionStore>) -> Self {
        let mut plans = HashMap::new();

        // Mock Plans Configuration
        plans.insert(FREE_PLAN_ID.to_string(), SubscriptionPlan {
            id: FREE_PLAN_ID.to_string(),
            name: "Free Tier".to_string(),
            features: vec!["basic_access".to_string()],
            quotas: HashMap::from([("users".to_string(), 5), ("api_calls".to_string(), 100)]),
            price_monthly: Some(0.0),
        });

        plans.insert(PRO_PLAN_ID.to_string(), SubscriptionPlan {
            id: PRO_PLAN_ID.to_string(),
            name: "Pro Tier".to_string(),
            features: vec!["basic_access".to_string(), "advanced_reporting".to_string(), "sso".to_string()],
            quotas: HashMap::from([("users".to_string(), 50), ("api_calls".to_string(), 10000)]),
            price_monthly: Some(29.99),
        });

        Self { store, plans }
    }

    pub async fn assign_plan(&self, tenant_id: Uuid, plan_id: &str) -> Result<TenantSubscription, AuthError> {
        if !self.plans.contains_key(plan_id) {
            return Err(AuthError::ValidationError { message: "Invalid plan ID".to_string() });
        }

        let sub = TenantSubscription {
            id: Uuid::new_v4(),
            tenant_id,
            plan_id: plan_id.to_string(),
            status: SubscriptionStatus::Active,
            start_date: Utc::now(),
            end_date: None,
            current_usage: sqlx::types::Json(HashMap::new()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.store.create(sub).await
    }

    pub async fn check_feature_access(&self, tenant_id: Uuid, feature: &str) -> Result<bool, AuthError> {
        let sub = self.store.get_by_tenant(tenant_id).await?;

        if let Some(subscription) = sub {
            if subscription.status != SubscriptionStatus::Active && subscription.status != SubscriptionStatus::Trialing {
                return Ok(false); // Only active/trialing subs get features
            }

            if let Some(plan) = self.plans.get(&subscription.plan_id) {
                return Ok(plan.features.contains(&feature.to_string()));
            }
        }

        Ok(false) // No subscription or unknown plan
    }

    pub async fn check_quota(&self, tenant_id: Uuid, resource: &str, requested_amount: i64) -> Result<bool, AuthError> {
        let sub = self.store.get_by_tenant(tenant_id).await?
            .ok_or(AuthError::ValidationError { message: "No subscription".to_string() })?;

        if let Some(plan) = self.plans.get(&sub.plan_id) {
            if let Some(limit) = plan.quotas.get(resource) {
                 let used = sub.current_usage.get(resource).unwrap_or(&0);
                 if used + requested_amount > *limit {
                     return Ok(false);
                 }
                 return Ok(true);
            }
            // If quota not defined for resource, assume unlimited? Or 0? Let's assume unlimited for now if not in map.
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn record_usage(&self, tenant_id: Uuid, resource: &str, amount: i64) -> Result<(), AuthError> {
        let sub = self.store.get_by_tenant(tenant_id).await?
             .ok_or(AuthError::ValidationError { message: "No subscription".to_string() })?;

        let mut usage_map = sub.current_usage.0.clone();
        let current = usage_map.entry(resource.to_string()).or_insert(0);
        *current += amount;

        self.store.update_usage(tenant_id, usage_map).await
    }
}
