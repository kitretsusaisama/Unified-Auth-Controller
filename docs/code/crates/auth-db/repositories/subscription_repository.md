# repositories/subscription_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/subscription_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::subscription_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ⚠️ **MEDIUM** - Billing and feature access

## Purpose

Implements the `SubscriptionStore` trait for MySQL/SQLite persistence, managing tenant subscriptions, plan assignments, and usage tracking.

### Problem It Solves

- Persists subscription data for SaaS billing
- Tracks usage for quota enforcement
- Manages subscription lifecycle
- Enables feature gating based on plan

---

## Detailed Code Breakdown

### Struct: `SubscriptionRepository`

**Purpose**: MySQL/SQLite implementation of `SubscriptionStore` trait

**Fields**:
- `pool`: `Pool<MySql>` - Database connection pool

---

### Method: `SubscriptionRepository::new()`

**Signature**: `pub fn new(pool: Pool<MySql>) -> Self`

**Purpose**: Constructor with connection pool

---

### Trait Implementation: `SubscriptionStore for SubscriptionRepository`

#### Method: `create()`

**Signature**: `async fn create(&self, sub: TenantSubscription) -> Result<TenantSubscription, AuthError>`

**Purpose**: Create new subscription record

**SQL Query**:
```sql
INSERT INTO tenant_subscriptions (
    id, tenant_id, plan_id, status, start_date, end_date, current_usage, created_at, updated_at
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
```

**Bindings**:
1. `id` - Subscription UUID
2. `tenant_id` - Tenant UUID
3. `plan_id` - Plan identifier (e.g., "free", "pro", "enterprise")
4. `status` - Subscription status (active, canceled, past_due, trialing)
5. `start_date` - Subscription start date
6. `end_date` - Subscription end date (optional)
7. `current_usage` - JSON usage counters
8. `created_at` - Creation timestamp
9. `updated_at` - Last update timestamp

**Returns**: Created subscription

**Example**:
```rust
let subscription = TenantSubscription {
    id: Uuid::new_v4(),
    tenant_id,
    plan_id: "pro".to_string(),
    status: SubscriptionStatus::Active,
    start_date: Utc::now(),
    end_date: None,
    current_usage: Json(hashmap!{
        "users" => 0,
        "api_calls_this_month" => 0,
    }),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

subscription_repo.create(subscription).await?;
```

---

#### Method: `get_by_tenant()`

**Signature**: `async fn get_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantSubscription>, AuthError>`

**Purpose**: Get active subscription for tenant

**SQL Query**:
```sql
SELECT * FROM tenant_subscriptions 
WHERE tenant_id = ? 
ORDER BY created_at DESC 
LIMIT 1
```

**Returns**: Most recent subscription (None if not found)

**Row Mapping**:
```rust
let row = sqlx::query(...)
    .bind(tenant_id.to_string())
    .fetch_optional(&self.pool)
    .await?;

if let Some(row) = row {
    // Manual mapping to handle UUID/String conversions
    Ok(Some(TenantSubscription {
        id: Uuid::parse_str(&row.try_get("id")?).unwrap_or_default(),
        tenant_id: Uuid::parse_str(&row.try_get("tenant_id")?).unwrap_or_default(),
        plan_id: row.try_get("plan_id")?,
        status: row.try_get("status")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date").unwrap_or(None),
        current_usage: row.try_get("current_usage")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    }))
}
```

**Usage**:
```rust
let subscription = subscription_repo.get_by_tenant(tenant_id).await?;
if let Some(sub) = subscription {
    println!("Plan: {}, Status: {:?}", sub.plan_id, sub.status);
}
```

---

#### Method: `update_usage()`

**Signature**: `async fn update_usage(&self, tenant_id: Uuid, usage: HashMap<String, i64>) -> Result<(), AuthError>`

**Purpose**: Update usage counters for subscription

**Process**:

1. **Fetch Current Subscription**
   ```rust
   let sub = self.get_by_tenant(tenant_id).await?
       .ok_or(AuthError::ValidationError { 
           message: "No subscription found".to_string() 
       })?;
   ```

2. **Update Usage**
   ```sql
   UPDATE tenant_subscriptions 
   SET current_usage = ?, updated_at = NOW() 
   WHERE id = ?
   ```

**Note**: This is a read-modify-write operation. For atomic increments, use database-specific JSON functions:

```sql
-- MySQL atomic increment
UPDATE tenant_subscriptions
SET current_usage = JSON_SET(
    current_usage,
    '$.users',
    JSON_EXTRACT(current_usage, '$.users') + 1
),
updated_at = NOW()
WHERE tenant_id = ?
```

**Usage**:
```rust
// After creating user
let mut usage = subscription.current_usage.0;
*usage.entry("users".to_string()).or_insert(0) += 1;
subscription_repo.update_usage(tenant_id, usage).await?;
```

---

## Database Schema

```sql
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

## Usage Tracking Patterns

### Pattern 1: Service-Level Tracking

```rust
pub async fn create_user(
    user_repo: &UserRepository,
    subscription_repo: &SubscriptionRepository,
    tenant_id: Uuid,
    request: CreateUserRequest,
) -> Result<User> {
    // Check quota
    let subscription = subscription_repo.get_by_tenant(tenant_id).await?
        .ok_or(AuthError::ValidationError { message: "No subscription".to_string() })?;
    
    let current_users = subscription.current_usage.0.get("users").copied().unwrap_or(0);
    let plan = get_plan(&subscription.plan_id).await?;
    let max_users = plan.quotas.get("users").copied().unwrap_or(0);
    
    if current_users >= max_users && max_users != -1 {
        return Err(AuthError::Conflict {
            message: "User limit reached".to_string()
        });
    }
    
    // Create user
    let user = user_repo.create(request, password_hash, tenant_id).await?;
    
    // Increment usage
    let mut usage = subscription.current_usage.0;
    *usage.entry("users".to_string()).or_insert(0) += 1;
    subscription_repo.update_usage(tenant_id, usage).await?;
    
    Ok(user)
}
```

---

### Pattern 2: Event-Driven Tracking

```rust
pub enum UsageEvent {
    UserCreated { tenant_id: Uuid },
    ApiCallMade { tenant_id: Uuid },
    SessionCreated { tenant_id: Uuid },
}

pub async fn handle_usage_event(
    event: UsageEvent,
    subscription_repo: &SubscriptionRepository,
) -> Result<()> {
    match event {
        UsageEvent::UserCreated { tenant_id } => {
            increment_usage(subscription_repo, tenant_id, "users", 1).await?;
        }
        UsageEvent::ApiCallMade { tenant_id } => {
            increment_usage(subscription_repo, tenant_id, "api_calls_this_month", 1).await?;
        }
        UsageEvent::SessionCreated { tenant_id } => {
            increment_usage(subscription_repo, tenant_id, "sessions_concurrent", 1).await?;
        }
    }
    Ok(())
}
```

---

## Subscription Lifecycle

### 1. Trial Creation
```rust
let subscription = TenantSubscription {
    id: Uuid::new_v4(),
    tenant_id,
    plan_id: "pro".to_string(),
    status: SubscriptionStatus::Trialing,
    start_date: Utc::now(),
    end_date: Some(Utc::now() + Duration::days(14)), // 14-day trial
    current_usage: Json(HashMap::new()),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};
```

### 2. Trial Conversion
```rust
pub async fn convert_trial(
    subscription_repo: &SubscriptionRepository,
    tenant_id: Uuid,
) -> Result<()> {
    let mut sub = subscription_repo.get_by_tenant(tenant_id).await?
        .ok_or(...)?;
    
    sub.status = SubscriptionStatus::Active;
    sub.end_date = None; // Remove trial end date
    sub.updated_at = Utc::now();
    
    // Update in database
    sqlx::query!(
        "UPDATE tenant_subscriptions SET status = ?, end_date = NULL, updated_at = ? WHERE id = ?",
        "active",
        sub.updated_at,
        sub.id.to_string()
    ).execute(&pool).await?;
    
    Ok(())
}
```

### 3. Cancellation
```rust
pub async fn cancel_subscription(
    subscription_repo: &SubscriptionRepository,
    tenant_id: Uuid,
) -> Result<()> {
    let mut sub = subscription_repo.get_by_tenant(tenant_id).await?
        .ok_or(...)?;
    
    sub.status = SubscriptionStatus::Canceled;
    sub.end_date = Some(calculate_billing_period_end(&sub));
    sub.updated_at = Utc::now();
    
    // Update
    sqlx::query!(
        "UPDATE tenant_subscriptions SET status = ?, end_date = ?, updated_at = ? WHERE id = ?",
        "canceled",
        sub.end_date,
        sub.updated_at,
        sub.id.to_string()
    ).execute(&pool).await?;
    
    Ok(())
}
```

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_create_subscription(pool: MySqlPool) {
    let repo = SubscriptionRepository::new(pool);
    
    let subscription = TenantSubscription {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        plan_id: "pro".to_string(),
        status: SubscriptionStatus::Active,
        start_date: Utc::now(),
        end_date: None,
        current_usage: Json(HashMap::new()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let created = repo.create(subscription.clone()).await.unwrap();
    assert_eq!(created.plan_id, "pro");
}

#[sqlx::test]
async fn test_update_usage(pool: MySqlPool) {
    let repo = SubscriptionRepository::new(pool);
    let tenant_id = Uuid::new_v4();
    
    // Create subscription
    let sub = create_test_subscription(tenant_id);
    repo.create(sub).await.unwrap();
    
    // Update usage
    let mut usage = HashMap::new();
    usage.insert("users".to_string(), 5);
    repo.update_usage(tenant_id, usage).await.unwrap();
    
    // Verify
    let updated = repo.get_by_tenant(tenant_id).await.unwrap().unwrap();
    assert_eq!(updated.current_usage.0.get("users"), Some(&5));
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database operations |
| `uuid` | Identifiers |
| `anyhow` | Error handling |
| `async-trait` | Async trait support |

### Internal Dependencies

- [auth-core/models/subscription.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/subscription.md) - Subscription models
- [auth-core/services/subscription_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - SubscriptionStore trait

---

## Related Files

- [models/subscription.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/subscription.md) - Subscription models
- [services/subscription_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Subscription service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 92  
**Security Level**: MEDIUM
