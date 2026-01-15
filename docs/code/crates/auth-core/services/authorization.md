# services/authorization.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/authorization.rs`  
**Crate**: `auth-core`  
**Module**: `services::authorization`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - Access control foundation

## Purpose

Defines the authorization service interface for Role-Based Access Control (RBAC) and Attribute-Based Access Control (ABAC), providing the foundation for fine-grained access control decisions.

### Problem It Solves

- Authorization decision making
- RBAC implementation
- ABAC policy evaluation
- Role management
- Permission checking

---

## Detailed Code Breakdown

### Trait: `AuthorizationProvider`

**Purpose**: Core authorization interface

**Methods**:

```rust
async fn authorize(&self, context: AuthzContext) -> Result<AuthzDecision, AuthError>;
async fn create_role(&self, role: CreateRoleRequest) -> Result<Role, AuthError>;
async fn assign_role(&self, assignment: RoleAssignment) -> Result<(), AuthError>;
async fn evaluate_policy(&self, policy: Policy, context: Context) -> Result<Decision, AuthError>;
```

---

### Struct: `AuthzContext`

**Purpose**: Authorization request context

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Uuid` | User making request |
| `tenant_id` | `Uuid` | Tenant context |
| `resource` | `String` | Resource being accessed |
| `action` | `String` | Action being performed |
| `attributes` | `serde_json::Value` | Additional context |

**Example**:
```rust
let context = AuthzContext {
    user_id,
    tenant_id,
    resource: "documents".to_string(),
    action: "read".to_string(),
    attributes: json!({
        "document_owner": owner_id,
        "document_status": "published"
    }),
};
```

---

### Struct: `AuthzDecision`

**Purpose**: Authorization decision result

**Fields**:
- `allowed`: `bool` - Whether access is granted
- `reason`: `String` - Explanation for decision
- `conditions`: `Vec<String>` - Additional conditions

**Example**:
```rust
AuthzDecision {
    allowed: true,
    reason: "User has 'documents:read' permission".to_string(),
    conditions: vec!["Must be within business hours".to_string()],
}
```

---

### Struct: `RoleAssignment`

**Purpose**: Role assignment request

**Fields**:
- `user_id`: User to assign role
- `tenant_id`: Tenant context
- `role_id`: Role to assign
- `expires_at`: Optional expiration

---

### Struct: `Policy`

**Purpose**: ABAC policy definition

**Fields**:
- `id`: Policy identifier
- `name`: Policy name
- `rules`: JSON policy rules

---

### Struct: `Context`

**Purpose**: Policy evaluation context

**Fields**:
- `user_id`: User
- `tenant_id`: Tenant
- `attributes`: Additional attributes

---

### Struct: `Decision`

**Purpose**: Policy evaluation result

**Fields**:
- `permit`: Whether to allow
- `obligations`: Actions that must be taken

---

## Authorization Patterns

### Pattern 1: RBAC Authorization

```rust
pub async fn check_permission(
    user_id: Uuid,
    tenant_id: Uuid,
    permission: &str,
    authz: &dyn AuthorizationProvider,
) -> Result<bool> {
    let context = AuthzContext {
        user_id,
        tenant_id,
        resource: permission.split(':').nth(0).unwrap_or("").to_string(),
        action: permission.split(':').nth(1).unwrap_or("").to_string(),
        attributes: json!({}),
    };
    
    let decision = authz.authorize(context).await?;
    Ok(decision.allowed)
}
```

---

### Pattern 2: ABAC Authorization

```rust
pub async fn check_document_access(
    user_id: Uuid,
    tenant_id: Uuid,
    document_id: Uuid,
    action: &str,
    authz: &dyn AuthorizationProvider,
) -> Result<bool> {
    let document = get_document(document_id).await?;
    
    let context = AuthzContext {
        user_id,
        tenant_id,
        resource: "documents".to_string(),
        action: action.to_string(),
        attributes: json!({
            "document_id": document_id,
            "document_owner": document.owner_id,
            "document_status": document.status,
            "user_department": get_user_department(user_id).await?,
        }),
    };
    
    let decision = authz.authorize(context).await?;
    Ok(decision.allowed)
}
```

---

### Pattern 3: Temporary Role Assignment

```rust
pub async fn grant_temporary_access(
    user_id: Uuid,
    tenant_id: Uuid,
    role_id: Uuid,
    duration: Duration,
    authz: &dyn AuthorizationProvider,
) -> Result<()> {
    let assignment = RoleAssignment {
        user_id,
        tenant_id,
        role_id,
        expires_at: Some(Utc::now() + duration),
    };
    
    authz.assign_role(assignment).await
}
```

---

## Usage Examples

### Example 1: Middleware Authorization

```rust
pub async fn authz_middleware(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let method = req.method();
    
    let context = AuthzContext {
        user_id,
        tenant_id,
        resource: extract_resource(path),
        action: method_to_action(method),
        attributes: json!({}),
    };
    
    let decision = state.authz_provider.authorize(context).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !decision.allowed {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

---

### Example 2: Service-Level Authorization

```rust
impl DocumentService {
    pub async fn delete_document(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        document_id: Uuid,
    ) -> Result<()> {
        // Check authorization
        let context = AuthzContext {
            user_id,
            tenant_id,
            resource: "documents".to_string(),
            action: "delete".to_string(),
            attributes: json!({ "document_id": document_id }),
        };
        
        let decision = self.authz.authorize(context).await?;
        if !decision.allowed {
            return Err(AuthError::AuthorizationDenied {
                permission: "documents:delete".to_string(),
                resource: document_id.to_string(),
            });
        }
        
        // Perform deletion
        self.repo.delete(document_id).await
    }
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `uuid` | Identifiers |
| `serde_json` | Attributes |
| `async-trait` | Async trait support |

### Internal Dependencies

- [models/role.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role model
- [error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [services/role_service.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/services/role_service.md) - Role service
- [models/permission.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/permission.md) - Permission model

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 67  
**Security Level**: CRITICAL
