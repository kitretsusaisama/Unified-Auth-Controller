# models/subscription.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/subscription.rs`  
**Crate**: `auth-core`  
**Module**: `models::subscription`  
**Layer**: Domain  
**Security-Critical**: ⚠️ **MEDIUM** - Billing and feature access

## Purpose

Defines subscription models for SaaS pricing tiers, enabling feature-gated access and usage tracking.

### Problem It Solves

- Implements SaaS pricing tiers (Free, Pro, Enterprise)
- Enforces feature limits and quotas
- Tracks usage for billing
- Manages subscription lifecycle

---

## Detailed Code Breakdown

### Struct: `SubscriptionPlan`

**Purpose**: Defines a subscription tier with features and quotas

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Plan identifier (e.g., "free", "pro", "enterprise") |
| `name` | `String` | Plan display name |
| `features` | `Vec<String>` | Enabled features |
| `quotas` | `HashMap<String, i64>` | Resource limits |
| `price_monthly` | `Option<f64>` | Monthly price (None for free) |

**Example Plans**:

```rust
// Free Plan
SubscriptionPlan {
    id: "free".to_string(),
    name: "Free".to_string(),
    features: vec![
        "basic_auth".to_string(),
        "password_login".to_string(),
    ],
    quotas: hashmap!{
        "users" => 5,
        "api_calls_per_month" => 1000,
        "sessions_concurrent" => 10,
    },
    price_monthly: None,
}

// Pro Plan
SubscriptionPlan {
    id: "pro".to_string(),
    name: "Professional".to_string(),
    features: vec![
        "basic_auth".to_string(),
        "password_login".to_string(),
        "oauth".to_string(),
        "mfa".to_string(),
        "custom_branding".to_string(),
    ],
    quotas: hashmap!{
        "users" => 100,
        "api_calls_per_month" => 50000,
        "sessions_concurrent" => 100,
        "tenants" => 3,
    },
    price_monthly: Some(99.0),
}

// Enterprise Plan
SubscriptionPlan {
    id: "enterprise".to_string(),
    name: "Enterprise".to_string(),
    features: vec![
        "basic_auth".to_string(),
        "password_login".to_string(),
        "oauth".to_string(),
        "oidc".to_string(),
        "saml".to_string(),
        "scim".to_string(),
        "mfa".to_string(),
        "custom_branding".to_string(),
        "sso".to_string(),
        "audit_logs".to_string(),
        "advanced_security".to_string(),
    ],
    quotas: hashmap!{
        "users" => -1,  // Unlimited
        "api_calls_per_month" => -1,
        "sessions_concurrent" => -1,
        "tenants" => -1,
    },
    price_monthly: Some(999.0),
}
```

---

### Struct: `TenantSubscription`

**Purpose**: Links a tenant to a subscription plan with usage tracking

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Subscription identifier |
| `tenant_id` | `Uuid` | Tenant this subscription belongs to |
| `plan_id` | `String` | Plan identifier (e.g., "pro") |
| `status` | `SubscriptionStatus` | Subscription status |
| `start_date` | `DateTime<Utc>` | Subscription start date |
| `end_date` | `Option<DateTime<Utc>>` | Subscription end date (None for active) |
| `current_usage` | `Json<HashMap<String, i64>>` | Current usage counters |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `updated_at` | `DateTime<Utc>` | Last update timestamp |

**Usage Tracking Example**:
```json
{
  "users": 45,
  "api_calls_this_month": 12500,
  "sessions_concurrent": 23,
  "tenants": 2
}
```

---

### Enum: `SubscriptionStatus`

**Purpose**: Subscription lifecycle states

**Variants**:

1. **`Active`**: Subscription is active
   - All features available
   - Billing current

2. **`Canceled`**: Subscription canceled
   - Access until end_date
   - No renewal

3. **`PastDue`**: Payment failed
   - Grace period active
   - Limited access
   - Requires payment update

4. **`Trialing`**: Free trial period
   - Full access
   - No payment required yet
   - Converts to paid or canceled

**Database Mapping**: Stored as lowercase string ("active", "canceled", "past_due", "trialing")

---

### SQLx Type Implementation

**Purpose**: Custom SQLx type for SubscriptionStatus enum

**Implementation**:

```rust
impl sqlx::Type<sqlx::MySql> for SubscriptionStatus {
    fn type_info() -> sqlx::mysql::MySqlTypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::MySql> for SubscriptionStatus {
    fn decode(value: sqlx::mysql::MySqlValueRef<'r>) 
        -> Result<Self, sqlx::error::BoxDynError> 
    {
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
```

---

## Feature Gating

### Check Feature Access

```rust
pub async fn has_feature(tenant_id: Uuid, feature: &str) -> Result<bool> {
    // 1. Get tenant subscription
    let subscription = get_tenant_subscription(tenant_id).await?;
    
    // 2. Check subscription status
    if !matches!(subscription.status, SubscriptionStatus::Active | SubscriptionStatus::Trialing) {
        return Ok(false);
    }
    
    // 3. Get plan
    let plan = get_subscription_plan(&subscription.plan_id).await?;
    
    // 4. Check if feature is in plan
    Ok(plan.features.contains(&feature.to_string()))
}
```

**Usage**:
```rust
// Before allowing OAuth login
if !has_feature(tenant_id, "oauth").await? {
    return Err(AuthError::Unauthorized {
        message: "OAuth not available in your plan".to_string()
    });
}

// Before enabling custom branding
if !has_feature(tenant_id, "custom_branding").await? {
    return Err(AuthError::Unauthorized {
        message: "Upgrade to Pro for custom branding".to_string()
    });
}
```

---

## Quota Enforcement

### Check Quota

```rust
pub async fn check_quota(
    tenant_id: Uuid,
    resource: &str,
) -> Result<bool> {
    // 1. Get subscription
    let subscription = get_tenant_subscription(tenant_id).await?;
    let plan = get_subscription_plan(&subscription.plan_id).await?;
    
    // 2. Get quota limit
    let limit = plan.quotas.get(resource).copied().unwrap_or(0);
    
    // 3. Check for unlimited (-1)
    if limit == -1 {
        return Ok(true);
    }
    
    // 4. Get current usage
    let usage = subscription.current_usage.0.get(resource).copied().unwrap_or(0);
    
    // 5. Compare
    Ok(usage < limit)
}
```

**Usage**:
```rust
// Before creating user
if !check_quota(tenant_id, "users").await? {
    return Err(AuthError::Conflict {
        message: "User limit reached. Upgrade your plan.".to_string()
    });
}

// Before API call
if !check_quota(tenant_id, "api_calls_per_month").await? {
    return Err(AuthError::RateLimitExceeded {
        limit: get_quota_limit(tenant_id, "api_calls_per_month").await?,
        window: "month".to_string(),
    });
}
```

---

## Usage Tracking

### Increment Usage

```rust
pub async fn increment_usage(
    tenant_id: Uuid,
    resource: &str,
    amount: i64,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE tenant_subscriptions
        SET current_usage = JSON_SET(
            current_usage,
            CONCAT('$.', ?),
            COALESCE(JSON_EXTRACT(current_usage, CONCAT('$.', ?)), 0) + ?
        ),
        updated_at = NOW()
        WHERE tenant_id = ?
        "#,
        resource,
        resource,
        amount,
        tenant_id.to_string()
    )
    .execute(&pool)
    .await?;
    
    Ok(())
}
```

**Usage**:
```rust
// After creating user
increment_usage(tenant_id, "users", 1).await?;

// After API call
increment_usage(tenant_id, "api_calls_this_month", 1).await?;

// After creating session
increment_usage(tenant_id, "sessions_concurrent", 1).await?;
```

### Reset Monthly Usage

```rust
pub async fn reset_monthly_usage() -> Result<()> {
    // Run monthly (cron job)
    sqlx::query!(
        r#"
        UPDATE tenant_subscriptions
        SET current_usage = JSON_SET(
            current_usage,
            '$.api_calls_this_month', 0
        )
        WHERE status = 'active'
        "#
    )
    .execute(&pool)
    .await?;
    
    Ok(())
}
```

---

## Subscription Lifecycle

### Trial to Paid

```rust
pub async fn convert_trial_to_paid(
    tenant_id: Uuid,
    payment_method: &str,
) -> Result<()> {
    // 1. Get subscription
    let mut subscription = get_tenant_subscription(tenant_id).await?;
    
    // 2. Validate trial status
    if subscription.status != SubscriptionStatus::Trialing {
        return Err(AuthError::ValidationError {
            message: "Not in trial period".to_string()
        });
    }
    
    // 3. Process payment
    process_payment(tenant_id, payment_method).await?;
    
    // 4. Update status
    subscription.status = SubscriptionStatus::Active;
    subscription.start_date = Utc::now();
    subscription.end_date = None;
    
    update_subscription(subscription).await?;
    
    Ok(())
}
```

### Upgrade Plan

```rust
pub async fn upgrade_plan(
    tenant_id: Uuid,
    new_plan_id: &str,
) -> Result<()> {
    // 1. Get current subscription
    let mut subscription = get_tenant_subscription(tenant_id).await?;
    
    // 2. Validate upgrade (can't downgrade mid-cycle)
    let current_plan = get_subscription_plan(&subscription.plan_id).await?;
    let new_plan = get_subscription_plan(new_plan_id).await?;
    
    if new_plan.price_monthly < current_plan.price_monthly {
        return Err(AuthError::ValidationError {
            message: "Cannot downgrade mid-cycle".to_string()
        });
    }
    
    // 3. Calculate prorated charge
    let prorated_amount = calculate_proration(&subscription, &new_plan).await?;
    
    // 4. Charge difference
    charge_customer(tenant_id, prorated_amount).await?;
    
    // 5. Update subscription
    subscription.plan_id = new_plan_id.to_string();
    update_subscription(subscription).await?;
    
    Ok(())
}
```

### Cancel Subscription

```rust
pub async fn cancel_subscription(tenant_id: Uuid) -> Result<()> {
    let mut subscription = get_tenant_subscription(tenant_id).await?;
    
    // Set end date to end of current billing period
    subscription.status = SubscriptionStatus::Canceled;
    subscription.end_date = Some(calculate_billing_period_end(&subscription));
    
    update_subscription(subscription).await?;
    
    Ok(())
}
```

---

## Database Schema

```sql
CREATE TABLE subscription_plans (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    features JSON NOT NULL,
    quotas JSON NOT NULL,
    price_monthly DECIMAL(10, 2),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE tenant_subscriptions (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL UNIQUE,
    plan_id VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'trialing',
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NULL,
    current_usage JSON DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (plan_id) REFERENCES subscription_plans(id),
    INDEX idx_status (status),
    INDEX idx_plan_id (plan_id)
);
```

---

## Security Considerations

### Quota Bypass Prevention

**Always check server-side**:
```rust
// DON'T: Client can bypass
if (clientSideQuotaCheck()) {
    createUser();
}

// DO: Server enforces
if !check_quota(tenant_id, "users").await? {
    return Err(AuthError::Conflict { ... });
}
```

### Usage Tracking Integrity

**Atomic increments**:
```sql
-- Atomic JSON update
UPDATE tenant_subscriptions
SET current_usage = JSON_SET(current_usage, '$.users', 
    COALESCE(JSON_EXTRACT(current_usage, '$.users'), 0) + 1
)
WHERE tenant_id = ?
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `serde` | JSON serialization |
| `uuid` | Unique identifiers |
| `chrono` | Timestamps |
| `sqlx` | Database types |

### Internal Dependencies

- [models/tenant.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/tenant.rs) - Tenant entity

---

## Related Files

- [services/subscription_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Subscription operations
- [repositories/subscription_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories) - Subscription persistence

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 72  
**Security Level**: MEDIUM
