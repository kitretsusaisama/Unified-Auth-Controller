# services/role_service.rs

## File Metadata

**File Path**: `crates/auth-core/src/services/role_service.rs`  
**Crate**: `auth-core`  
**Module**: `services::role_service`  
**Layer**: Domain (Business Logic)  
**Security-Critical**: ✅ **YES** - RBAC foundation

## Purpose

Manages role creation, updates, and validation for Role-Based Access Control (RBAC), ensuring role name uniqueness and proper hierarchy.

### Problem It Solves

- Role lifecycle management
- Role name uniqueness enforcement
- Role hierarchy validation
- Permission assignment
- RBAC foundation

---

## Detailed Code Breakdown

### Trait: `RoleStore`

**Purpose**: Persistence abstraction for roles

**Methods**:
```rust
async fn create(&self, role: Role) -> Result<Role, AuthError>;
async fn update(&self, id: Uuid, req: UpdateRoleRequest) -> Result<Role, AuthError>;
async fn delete(&self, id: Uuid) -> Result<(), AuthError>;
async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, AuthError>;
async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Role>, AuthError>;
async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<Role>, AuthError>;
```

---

### Struct: `RoleService`

**Purpose**: Role business logic

**Fields**:
- `store`: `Arc<dyn RoleStore>` - Persistence layer

---

### Method: `RoleService::new()`

**Signature**: `pub fn new(store: Arc<dyn RoleStore>) -> Self`

**Purpose**: Constructor with dependency injection

---

### Method: `create_role()`

**Signature**: `pub async fn create_role(&self, tenant_id: Uuid, req: CreateRoleRequest) -> Result<Role, AuthError>`

**Purpose**: Create new role with validation

**Process**:

#### 1. Check Name Uniqueness
```rust
if let Some(_) = self.store.find_by_name(tenant_id, &req.name).await? {
    return Err(AuthError::ValidationError { 
        message: "Role with this name already exists".to_string() 
    });
}
```

**Prevents**: Duplicate role names within tenant

#### 2. Create Role
```rust
let role = Role {
    id: Uuid::new_v4(),
    tenant_id,
    name: req.name,
    description: req.description,
    parent_role_id: req.parent_role_id,
    is_system_role: false, // User-created roles are never system roles
    permissions: Json(req.permissions),
    constraints: Json(req.constraints.unwrap_or_default()),
    created_at: Utc::now(),
    updated_at: None,
};
```

#### 3. Persist
```rust
self.store.create(role).await
```

**Example**:
```rust
let request = CreateRoleRequest {
    name: "Content Manager".to_string(),
    description: Some("Manages content".to_string()),
    parent_role_id: None,
    permissions: vec![
        "content:read:tenant".to_string(),
        "content:write:tenant".to_string(),
    ],
    constraints: Some(HashMap::new()),
};

let role = role_service.create_role(tenant_id, request).await?;
```

---

## Role Hierarchy

### Parent-Child Relationships

**Example Hierarchy**:
```
Admin (system role)
├── Content Manager
│   ├── Editor
│   └── Reviewer
└── User Manager
    └── Support Agent
```

**Implementation**:
```rust
// Create parent role
let admin = role_service.create_role(tenant_id, CreateRoleRequest {
    name: "Admin".to_string(),
    parent_role_id: None,
    permissions: vec!["*:*:*".to_string()],
    ..Default::default()
}).await?;

// Create child role
let content_manager = role_service.create_role(tenant_id, CreateRoleRequest {
    name: "Content Manager".to_string(),
    parent_role_id: Some(admin.id),
    permissions: vec!["content:*:tenant".to_string()],
    ..Default::default()
}).await?;
```

---

## Permission Inheritance

### Effective Permissions Calculation

```rust
pub async fn get_effective_permissions(
    role_id: Uuid,
    role_service: &RoleService,
) -> Result<Vec<String>> {
    let mut permissions = HashSet::new();
    let mut current_role_id = Some(role_id);
    
    // Traverse up the hierarchy
    while let Some(id) = current_role_id {
        let role = role_service.store.find_by_id(id).await?
            .ok_or(AuthError::ValidationError { 
                message: "Role not found".to_string() 
            })?;
        
        // Add role's permissions
        for perm in role.permissions.0 {
            permissions.insert(perm);
        }
        
        // Move to parent
        current_role_id = role.parent_role_id;
    }
    
    Ok(permissions.into_iter().collect())
}
```

**Example**:
```
Editor role: ["content:write:own"]
  ↓ inherits from
Content Manager: ["content:read:tenant"]
  ↓ inherits from
Admin: ["*:*:*"]

Effective permissions for Editor:
["content:write:own", "content:read:tenant", "*:*:*"]
```

---

## Validation Patterns

### Pattern 1: Circular Hierarchy Prevention

```rust
pub async fn validate_no_circular_dependency(
    role_id: Uuid,
    parent_id: Uuid,
    role_service: &RoleService,
) -> Result<(), AuthError> {
    let mut current = Some(parent_id);
    let mut visited = HashSet::new();
    
    while let Some(id) = current {
        // Check if we've reached the original role
        if id == role_id {
            return Err(AuthError::ValidationError {
                message: "Circular role hierarchy detected".to_string(),
            });
        }
        
        // Check for loops
        if !visited.insert(id) {
            return Err(AuthError::ValidationError {
                message: "Loop detected in role hierarchy".to_string(),
            });
        }
        
        // Move to next parent
        let role = role_service.store.find_by_id(id).await?;
        current = role.and_then(|r| r.parent_role_id);
    }
    
    Ok(())
}
```

### Pattern 2: System Role Protection

```rust
pub async fn delete_role(
    role_id: Uuid,
    role_service: &RoleService,
) -> Result<(), AuthError> {
    let role = role_service.store.find_by_id(role_id).await?
        .ok_or(AuthError::ValidationError { 
            message: "Role not found".to_string() 
        })?;
    
    // Prevent deletion of system roles
    if role.is_system_role {
        return Err(AuthError::ValidationError {
            message: "Cannot delete system role".to_string(),
        });
    }
    
    role_service.store.delete(role_id).await
}
```

---

## Usage Examples

### Example 1: Create Role Hierarchy

```rust
// Create organization structure
let admin = role_service.create_role(tenant_id, CreateRoleRequest {
    name: "Admin".to_string(),
    permissions: vec!["*:*:*".to_string()],
    ..Default::default()
}).await?;

let manager = role_service.create_role(tenant_id, CreateRoleRequest {
    name: "Manager".to_string(),
    parent_role_id: Some(admin.id),
    permissions: vec![
        "users:read:tenant".to_string(),
        "reports:read:tenant".to_string(),
    ],
    ..Default::default()
}).await?;

let employee = role_service.create_role(tenant_id, CreateRoleRequest {
    name: "Employee".to_string(),
    parent_role_id: Some(manager.id),
    permissions: vec!["profile:read:own".to_string()],
    ..Default::default()
}).await?;
```

### Example 2: Role Assignment

```rust
pub async fn assign_role_to_user(
    user_id: Uuid,
    role_id: Uuid,
    pool: &MySqlPool,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)",
        user_id.to_string(),
        role_id.to_string()
    ).execute(pool).await?;
    
    Ok(())
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_create_role() {
    let store = Arc::new(MockRoleStore::new());
    let service = RoleService::new(store);
    
    let request = CreateRoleRequest {
        name: "Test Role".to_string(),
        permissions: vec!["test:read:tenant".to_string()],
        ..Default::default()
    };
    
    let role = service.create_role(tenant_id, request).await.unwrap();
    assert_eq!(role.name, "Test Role");
}

#[tokio::test]
async fn test_duplicate_role_name_fails() {
    let store = Arc::new(MockRoleStore::new());
    let service = RoleService::new(store);
    
    // Create first role
    service.create_role(tenant_id, CreateRoleRequest {
        name: "Admin".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    // Try to create duplicate
    let result = service.create_role(tenant_id, CreateRoleRequest {
        name: "Admin".to_string(),
        ..Default::default()
    }).await;
    
    assert!(matches!(result, Err(AuthError::ValidationError { .. })));
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `uuid` | Role identifiers |
| `chrono` | Timestamps |
| `sqlx` | JSON types |
| `async-trait` | Async trait support |

### Internal Dependencies

- [models/role.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role model
- [repositories/role_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/role_repository.md) - Role persistence

---

## Related Files

- [models/role.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role model
- [repositories/role_repository.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-db/repositories/role_repository.md) - Role repository
- [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs) - Authorization service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 49  
**Security Level**: CRITICAL
