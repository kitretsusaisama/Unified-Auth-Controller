# services/subscription_service.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/subscription_service.rs`  
**Crate**: `auth-core`  
**Module**: `services::subscription_service`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ⚠️ **MEDIUM** - Feature gating and quota enforcement

## Purpose

Manages subscription plans, feature access control, and usage quota enforcement for multi-tenant SaaS billing and feature gating.

### Problem It Solves

- Subscription plan management
- Feature access control (feature flags)
- Usage quota enforcement
- Billing tier differentiation
- Resource usage tracking

---

## Detailed Code Breakdown

### Trait: `SubscriptionStore`

**Purpose**: Persistence abstraction for subscriptions

**Methods**:
```rust
async fn create(&self, sub: TenantSubscription) -> Result<TenantSubscription, AuthError>;
async fn get_by_tenant(&self, tenant_id: Uuid) -> Result<Option<TenantSubscription>, AuthError>;
async fn update_usage(&self, tenant_id: Uuid, usage: HashMap<String, i64>) -> Result<(), AuthError>;
```

---

### Struct: `SubscriptionService`

**Purpose**: Subscription business logic

**Fields**:
- `store`: `Arc<dyn SubscriptionStore>` - Persistence layer
- `plans`: `HashMap<String, SubscriptionPlan>` - Available plans

---

### Method: `SubscriptionService::new()`

**Purpose**: Initialize service with predefined plans

**Plans Configuration**:

#### Free Tier
```rust
SubscriptionPlan {
    id: "free",
    name: "Free Tier",
    features: vec!["basic_access"],
    quotas: HashMap::from([
        ("users", 5),
        ("api_calls", 100)
    ]),
    price_monthly: Some(0.0),
}
```

#### Pro Tier
```rust
SubscriptionPlan {
    id: "pro",
    name: "Pro Tier",
    features: vec!["basic_access", "advanced_reporting", "sso"],
    quotas: HashMap::from([
        ("users", 50),
        ("api_calls", 10000)
    ]),
    price_monthly: Some(29.99),
}
```

---

### Method: `assign_plan()`

**Signature**: `pub async fn assign_plan(&self, tenant_id: Uuid, plan_id: &str) -> Result<TenantSubscription, AuthError>`

**Purpose**: Assign subscription plan to tenant

**Process**:

1. **Validate Plan**
   ```rust
   if !self.plans.contains_key(plan_id) {
       return Err(AuthError::ValidationError { 
           message: "Invalid plan ID".to_string() 
       });
   }
   ```

2. **Create Subscription**
   ```rust
   let sub = TenantSubscription {
       id: Uuid::new_v4(),
       tenant_id,
       plan_id: plan_id.to_string(),
       status: SubscriptionStatus::Active,
       start_date: Utc::now(),
       end_date: None,
       current_usage: Json(HashMap::new()),
       created_at: Utc::now(),
       updated_at: Utc::now(),
   };
   ```

3. **Persist**
   ```rust
   self.store.create(sub).await
   ```

**Example**:
```rust
// Assign Pro plan to tenant
let subscription = subscription_service
    .assign_plan(tenant_id, "pro")
    .await?;
```

---

### Method: `check_feature_access()`

**Signature**: `pub async fn check_feature_access(&self, tenant_id: Uuid, feature: &str) -> Result<bool, AuthError>`

**Purpose**: Check if tenant has access to feature

**Logic**:

1. **Get Subscription**
   ```rust
   let sub = self.store.get_by_tenant(tenant_id).await?;
   ```

2. **Check Status**
   ```rust
   if subscription.status != SubscriptionStatus::Active 
       && subscription.status != SubscriptionStatus::Trialing {
       return Ok(false);
   }
   ```

3. **Check Feature**
   ```rust
   if let Some(plan) = self.plans.get(&subscription.plan_id) {
       return Ok(plan.features.contains(&feature.to_string()));
   }
   ```

**Usage**:
```rust
// Check if tenant has SSO feature
if subscription_service.check_feature_access(tenant_id, "sso").await? {
    // Enable SSO
} else {
    return Err(AuthError::FeatureNotAvailable {
        feature: "sso".to_string(),
    });
}
```

---

### Method: `check_quota()`

**Signature**: `pub async fn check_quota(&self, tenant_id: Uuid, resource: &str, requested_amount: i64) -> Result<bool, AuthError>`

**Purpose**: Check if quota allows requested resource usage

**Logic**:

1. **Get Subscription**
   ```rust
   let sub = self.store.get_by_tenant(tenant_id).await?
       .ok_or(AuthError::ValidationError { message: "No subscription".to_string() })?;
   ```

2. **Get Plan Limit**
   ```rust
   if let Some(plan) = self.plans.get(&sub.plan_id) {
       if let Some(limit) = plan.quotas.get(resource) {
           let used = sub.current_usage.get(resource).unwrap_or(&0);
           if used + requested_amount > *limit {
               return Ok(false); // Quota exceeded
           }
           return Ok(true); // Within quota
       }
   }
   ```

**Example**:
```rust
// Check if tenant can create user
if !subscription_service.check_quota(tenant_id, "users", 1).await? {
    return Err(AuthError::QuotaExceeded {
        resource: "users".to_string(),
        limit: 5,
        current: 5,
    });
}

// Create user
user_service.create_user(request).await?;

// Record usage
subscription_service.record_usage(tenant_id, "users", 1).await?;
```

---

### Method: `record_usage()`

**Signature**: `pub async fn record_usage(&self, tenant_id: Uuid, resource: &str, amount: i64) -> Result<(), AuthError>`

**Purpose**: Increment usage counter

**Process**:

1. **Get Current Subscription**
   ```rust
   let sub = self.store.get_by_tenant(tenant_id).await?
       .ok_or(AuthError::ValidationError { message: "No subscription".to_string() })?;
   ```

2. **Update Usage**
   ```rust
   let mut usage_map = sub.current_usage.0.clone();
   let current = usage_map.entry(resource.to_string()).or_insert(0);
   *current += amount;
   ```

3. **Persist**
   ```rust
   self.store.update_usage(tenant_id, usage_map).await
   ```

---

## Feature Gating Patterns

### Pattern 1: Middleware

```rust
pub async fn feature_gate_middleware(
    State(state): State<AppState>,
    Extension(tenant_id): Extension<Uuid>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if tenant has required feature
    let has_feature = state
        .subscription_service
        .check_feature_access(tenant_id, "advanced_reporting")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !has_feature {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }
    
    Ok(next.run(req).await)
}
```

### Pattern 2: Decorator

```rust
pub async fn with_feature_check<F, T>(
    tenant_id: Uuid,
    feature: &str,
    subscription_service: &SubscriptionService,
    operation: F,
) -> Result<T, AuthError>
where
    F: Future<Output = Result<T, AuthError>>,
{
    if !subscription_service.check_feature_access(tenant_id, feature).await? {
        return Err(AuthError::FeatureNotAvailable {
            feature: feature.to_string(),
        });
    }
    
    operation.await
}
```

### Pattern 3: Service-Level

```rust
impl UserService {
    pub async fn create_user(
        &self,
        tenant_id: Uuid,
        request: CreateUserRequest,
    ) -> Result<User, AuthError> {
        // Check quota
        if !self.subscription_service.check_quota(tenant_id, "users", 1).await? {
            return Err(AuthError::QuotaExceeded {
                resource: "users".to_string(),
                limit: 5,
                current: 5,
            });
        }
        
        // Create user
        let user = self.user_repo.create(request).await?;
        
        // Record usage
        self.subscription_service.record_usage(tenant_id, "users", 1).await?;
        
        Ok(user)
    }
}
```

---

## Subscription Lifecycle

### 1. Trial Creation
```rust
pub async fn start_trial(
    tenant_id: Uuid,
    subscription_service: &SubscriptionService,
) -> Result<TenantSubscription> {
    subscription_service.assign_plan(tenant_id, "pro").await?;
    
    // Update to trialing status
    sqlx::query!(
        "UPDATE tenant_subscriptions SET status = 'trialing', end_date = ? WHERE tenant_id = ?",
        Utc::now() + Duration::days(14),
        tenant_id.to_string()
    ).execute(&pool).await?;
    
    Ok(subscription)
}
```

### 2. Trial Conversion
```rust
pub async fn convert_trial(
    tenant_id: Uuid,
) -> Result<()> {
    sqlx::query!(
        "UPDATE tenant_subscriptions SET status = 'active', end_date = NULL WHERE tenant_id = ?",
        tenant_id.to_string()
    ).execute(&pool).await?;
    
    Ok(())
}
```

### 3. Downgrade
```rust
pub async fn downgrade_plan(
    tenant_id: Uuid,
    subscription_service: &SubscriptionService,
) -> Result<()> {
    // Check if current usage fits in free tier
    let subscription = subscription_service.store.get_by_tenant(tenant_id).await?;
    let current_users = subscription.current_usage.0.get("users").unwrap_or(&0);
    
    if *current_users > 5 {
        return Err(AuthError::ValidationError {
            message: "Cannot downgrade: too many users".to_string(),
        });
    }
    
    subscription_service.assign_plan(tenant_id, "free").await?;
    Ok(())
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_check_feature_access() {
    let store = Arc::new(MockSubscriptionStore::new());
    let service = SubscriptionService::new(store);
    
    // Assign Pro plan
    service.assign_plan(tenant_id, "pro").await.unwrap();
    
    // Check SSO feature (Pro only)
    assert!(service.check_feature_access(tenant_id, "sso").await.unwrap());
    
    // Check non-existent feature
    assert!(!service.check_feature_access(tenant_id, "nonexistent").await.unwrap());
}

#[tokio::test]
async fn test_quota_enforcement() {
    let store = Arc::new(MockSubscriptionStore::new());
    let service = SubscriptionService::new(store);
    
    // Assign Free plan (5 users max)
    service.assign_plan(tenant_id, "free").await.unwrap();
    
    // Record 5 users
    service.record_usage(tenant_id, "users", 5).await.unwrap();
    
    // Check if can add more
    assert!(!service.check_quota(tenant_id, "users", 1).await.unwrap());
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `uuid` | Identifiers |
| `chrono` | Timestamps |
| `anyhow` | Error handling |

### Internal Dependencies

- [models/subscription.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/subscription.md) - Subscription models
- [repositories/subscription_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/subscription_repository.md) - Persistence

---

## Related Files

- [models/subscription.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/subscription.md) - Subscription models
- [repositories/subscription_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/subscription_repository.md) - Subscription repository

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 114  
**Security Level**: MEDIUM
