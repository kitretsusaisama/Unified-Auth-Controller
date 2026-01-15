---
title: Authorization Engine Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Module Specification
crate: auth-core
---

# Authorization Engine Specification

> [!NOTE]
> **Module**: `auth-core::services::authorization`  
> **Responsibility**: RBAC and ABAC policy enforcement

---

## 1. Overview

The **Authorization Engine** enforces access control policies using Role-Based Access Control (RBAC) and Attribute-Based Access Control (ABAC). It evaluates whether a user has permission to perform an action on a resource.

---

## 2. Public API

### 2.1 Traits

```rust
#[async_trait]
pub trait AuthorizationProvider: Send + Sync {
    async fn authorize(&self, context: AuthzContext) 
        -> Result<AuthzDecision, AuthError>;
    
    async fn create_role(&self, role: CreateRoleRequest) 
        -> Result<Role, AuthError>;
    
    async fn assign_role(&self, assignment: RoleAssignment) 
        -> Result<(), AuthError>;
    
    async fn evaluate_policy(&self, policy: Policy, context: Context) 
        -> Result<Decision, AuthError>;
}
```

---

## 3. Models

### 3.1 RBAC Models

```rust
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub tenant_id: Uuid,
}

pub struct Permission {
    pub id: Uuid,
    pub resource: String,      // e.g., "users", "tenants"
    pub action: String,        // e.g., "read", "write", "delete"
}

pub struct RoleAssignment {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role_id: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
}
```

### 3.2 ABAC Models

```rust
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub rules: serde_json::Value,  // JSON policy rules
}

pub struct AuthzContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub resource: String,
    pub action: String,
    pub attributes: serde_json::Value,  // Context attributes
}

pub struct AuthzDecision {
    pub allowed: bool,
    pub reason: String,
    pub conditions: Vec<String>,
}
```

---

## 4. Operations

### 4.1 Authorize (RBAC)

**Method**: `authorize(context: AuthzContext) -> Result<AuthzDecision>`

**Flow**:
1. Fetch user's roles for tenant
2. Extract permissions from roles
3. Check if any permission matches (resource + action)
4. Return decision

**Example**:
```rust
let context = AuthzContext {
    user_id: user.id,
    tenant_id: tenant.id,
    resource: "users".to_string(),
    action: "read".to_string(),
    attributes: json!({}),
};

let decision = authz_engine.authorize(context).await?;
assert!(decision.allowed);
```

---

### 4.2 Evaluate Policy (ABAC)

**Method**: `evaluate_policy(policy: Policy, context: Context) -> Result<Decision>`

**Flow**:
1. Parse policy rules (JSON)
2. Evaluate rules against context attributes
3. Return permit/deny decision

**Policy Example**:
```json
{
  "rules": [
    {
      "effect": "allow",
      "conditions": [
        {"attribute": "time_of_day", "operator": "between", "values": ["09:00", "17:00"]},
        {"attribute": "ip_address", "operator": "in_range", "values": ["10.0.0.0/8"]}
      ]
    }
  ]
}
```

---

## 5. Security Considerations

- **Principle of Least Privilege**: Deny by default
- **Tenant Isolation**: All authorization scoped to tenant
- **Audit Logging**: All authorization decisions logged

---

**Document Status**: Active  
**Owner**: Engineering Team
