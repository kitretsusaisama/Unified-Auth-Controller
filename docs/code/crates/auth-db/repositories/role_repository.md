# repositories/role_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/role_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::role_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ✅ **YES** - RBAC foundation for authorization

## Purpose

Implements the `RoleStore` trait for MySQL/SQLite persistence, providing CRUD operations for roles with permission management and hierarchy support.

### Problem It Solves

- Persists role definitions and hierarchies
- Manages role-permission mappings
- Supports multi-tenancy for roles
- Enables RBAC (Role-Based Access Control)

---

## Detailed Code Breakdown

### Struct: `RoleRepository`

**Purpose**: MySQL/SQLite implementation of `RoleStore` trait

**Fields**:
- `pool`: `Pool<MySql>` - Database connection pool

---

### Method: `RoleRepository::new()`

**Signature**: `pub fn new(pool: Pool<MySql>) -> Self`

**Purpose**: Constructor with connection pool

---

### Trait Implementation: `RoleStore for RoleRepository`

#### Method: `create()`

**Signature**: `async fn create(&self, role: Role) -> Result<Role, AuthError>`

**Purpose**: Create new role in database

**SQL Query**:
```sql
INSERT INTO roles (
    id, tenant_id, name, description, parent_role_id, 
    is_system_role, permissions, constraints, created_at, updated_at
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

**Bindings**:
1. `id` - Role UUID
2. `tenant_id` - Tenant UUID (multi-tenancy)
3. `name` - Role name (e.g., "Admin", "User Manager")
4. `description` - Optional description
5. `parent_role_id` - Parent role for hierarchy (optional)
6. `is_system_role` - System-defined role flag
7. `permissions` - JSON array of permission codes
8. `constraints` - JSON object of ABAC constraints
9. `created_at` - Creation timestamp
10. `updated_at` - Last update timestamp

**Returns**: Created role

**Example**:
```rust
let role = Role {
    id: Uuid::new_v4(),
    tenant_id,
    name: "User Manager".to_string(),
    description: Some("Manages user accounts".to_string()),
    parent_role_id: Some(admin_role_id),
    is_system_role: false,
    permissions: Json(vec![
        "users:read:tenant".to_string(),
        "users:write:tenant".to_string(),
    ]),
    constraints: Json(HashMap::new()),
    created_at: Utc::now(),
    updated_at: None,
};

role_repo.create(role).await?;
```

---

#### Method: `update()`

**Signature**: `async fn update(&self, id: Uuid, req: UpdateRoleRequest) -> Result<Role, AuthError>`

**Purpose**: Update existing role

**Process**:

1. **Fetch Current Role**
   ```rust
   let mut current_role = self.find_by_id(id).await?
       .ok_or(AuthError::ValidationError { 
           message: "Role not found".to_string() 
       })?;
   ```

2. **Apply Updates** (partial update pattern)
   ```rust
   if let Some(name) = req.name { current_role.name = name; }
   if let Some(desc) = req.description { current_role.description = Some(desc); }
   if let Some(parent) = req.parent_role_id { current_role.parent_role_id = Some(parent); }
   if let Some(perms) = req.permissions { current_role.permissions = Json(perms); }
   if let Some(cons) = req.constraints { current_role.constraints = Json(cons); }
   current_role.updated_at = Some(Utc::now());
   ```

3. **Save to Database**
   ```sql
   UPDATE roles 
   SET name=?, description=?, parent_role_id=?, permissions=?, constraints=?, updated_at=?
   WHERE id=?
   ```

**Returns**: Updated role

---

#### Method: `delete()`

**Signature**: `async fn delete(&self, id: Uuid) -> Result<(), AuthError>`

**Purpose**: Delete role from database

**SQL Query**:
```sql
DELETE FROM roles WHERE id = ?
```

**Cascade Effects**:
- User-role assignments deleted (via foreign key)
- Child roles may need re-parenting

**Security**:
```rust
// Prevent deletion of system roles
if role.is_system_role {
    return Err(AuthError::ValidationError {
        message: "Cannot delete system role".to_string()
    });
}
```

---

#### Method: `find_by_id()`

**Signature**: `async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, AuthError>`

**Purpose**: Retrieve role by UUID

**SQL Query**:
```sql
SELECT * FROM roles WHERE id = ?
```

**Returns**: `Option<Role>` (None if not found)

---

#### Method: `find_by_tenant()`

**Signature**: `async fn find_by_tenant(&self, tenant_id: Uuid) -> Result<Vec<Role>, AuthError>`

**Purpose**: Get all roles for a tenant

**SQL Query**:
```sql
SELECT * FROM roles WHERE tenant_id = ?
```

**Returns**: Vector of all tenant roles

**Use Cases**:
- Role management UI
- Permission assignment
- Role hierarchy display

**Example**:
```rust
let roles = role_repo.find_by_tenant(tenant_id).await?;
for role in roles {
    println!("{}: {} permissions", role.name, role.permissions.0.len());
}
```

---

#### Method: `find_by_name()`

**Signature**: `async fn find_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<Role>, AuthError>`

**Purpose**: Find role by name within tenant

**SQL Query**:
```sql
SELECT * FROM roles WHERE tenant_id = ? AND name = ?
```

**Returns**: `Option<Role>`

**Use Cases**:
- Role name uniqueness validation
- Role lookup by name
- Assignment by role name

**Example**:
```rust
if role_repo.find_by_name(tenant_id, "Admin").await?.is_some() {
    return Err(AuthError::Conflict {
        message: "Role name already exists".to_string()
    });
}
```

---

## Database Schema

```sql
CREATE TABLE roles (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_role_id CHAR(36),
    is_system_role BOOLEAN DEFAULT FALSE,
    permissions JSON NOT NULL,
    constraints JSON DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    UNIQUE KEY unique_role_name_tenant (tenant_id, name),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_parent_role_id (parent_role_id),
    INDEX idx_is_system_role (is_system_role),
    
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_role_id) REFERENCES roles(id) ON DELETE SET NULL
);
```

---

## Role Hierarchy

### Parent-Child Relationships

**Example Hierarchy**:
```
SuperAdmin (system role)
├── TenantAdmin
│   ├── UserManager
│   └── ContentManager
└── Auditor
```

**Implementation**:
```rust
// Create hierarchy
let super_admin = create_role("SuperAdmin", None, true).await?;
let tenant_admin = create_role("TenantAdmin", Some(super_admin.id), false).await?;
let user_manager = create_role("UserManager", Some(tenant_admin.id), false).await?;
```

### Permission Inheritance

**Logic**:
```rust
pub async fn get_effective_permissions(
    role_id: Uuid,
    role_repo: &RoleRepository,
) -> Result<Vec<String>> {
    let mut permissions = Vec::new();
    let mut current_role_id = Some(role_id);
    
    // Traverse up the hierarchy
    while let Some(id) = current_role_id {
        let role = role_repo.find_by_id(id).await?
            .ok_or(AuthError::ValidationError { message: "Role not found".to_string() })?;
        
        // Add role's permissions
        permissions.extend(role.permissions.0.clone());
        
        // Move to parent
        current_role_id = role.parent_role_id;
    }
    
    // Deduplicate
    permissions.sort();
    permissions.dedup();
    
    Ok(permissions)
}
```

---

## System Roles

### Predefined Roles

```rust
pub async fn create_system_roles(
    tenant_id: Uuid,
    role_repo: &RoleRepository,
) -> Result<()> {
    // SuperAdmin
    role_repo.create(Role {
        id: Uuid::new_v4(),
        tenant_id,
        name: "SuperAdmin".to_string(),
        description: Some("Full system access".to_string()),
        parent_role_id: None,
        is_system_role: true,
        permissions: Json(vec!["*:*:*".to_string()]),
        constraints: Json(HashMap::new()),
        created_at: Utc::now(),
        updated_at: None,
    }).await?;
    
    // TenantAdmin
    role_repo.create(Role {
        id: Uuid::new_v4(),
        tenant_id,
        name: "TenantAdmin".to_string(),
        description: Some("Tenant administration".to_string()),
        parent_role_id: None,
        is_system_role: true,
        permissions: Json(vec![
            "users:*:tenant".to_string(),
            "roles:*:tenant".to_string(),
            "settings:*:tenant".to_string(),
        ]),
        constraints: Json(HashMap::new()),
        created_at: Utc::now(),
        updated_at: None,
    }).await?;
    
    Ok(())
}
```

---

## Security Considerations

### 1. Multi-Tenancy Isolation

**Enforcement**:
```rust
// Always filter by tenant
let role = role_repo.find_by_name(tenant_id, role_name).await?;

// Prevent cross-tenant role assignment
if role.tenant_id != user.tenant_id {
    return Err(AuthError::AuthorizationDenied { ... });
}
```

### 2. System Role Protection

**Validation**:
```rust
if role.is_system_role && !user.is_super_admin() {
    return Err(AuthError::AuthorizationDenied {
        permission: "roles:manage:global".to_string(),
        resource: "system_role".to_string(),
    });
}
```

### 3. Circular Hierarchy Prevention

**Check**:
```rust
pub fn has_circular_dependency(
    role_id: Uuid,
    parent_id: Uuid,
    role_repo: &RoleRepository,
) -> Result<bool> {
    let mut current = Some(parent_id);
    let mut visited = HashSet::new();
    
    while let Some(id) = current {
        if id == role_id {
            return Ok(true);  // Circular!
        }
        
        if !visited.insert(id) {
            return Ok(true);  // Loop detected
        }
        
        let role = role_repo.find_by_id(id).await?;
        current = role.and_then(|r| r.parent_role_id);
    }
    
    Ok(false)
}
```

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_create_role(pool: MySqlPool) {
    let repo = RoleRepository::new(pool);
    
    let role = Role {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        name: "Test Role".to_string(),
        description: Some("Test description".to_string()),
        parent_role_id: None,
        is_system_role: false,
        permissions: Json(vec!["users:read:tenant".to_string()]),
        constraints: Json(HashMap::new()),
        created_at: Utc::now(),
        updated_at: None,
    };
    
    let created = repo.create(role.clone()).await.unwrap();
    assert_eq!(created.name, "Test Role");
}

#[sqlx::test]
async fn test_role_name_uniqueness(pool: MySqlPool) {
    let repo = RoleRepository::new(pool);
    let tenant_id = Uuid::new_v4();
    
    // Create first role
    let role1 = create_test_role(tenant_id, "Admin");
    repo.create(role1).await.unwrap();
    
    // Try to create duplicate
    let role2 = create_test_role(tenant_id, "Admin");
    let result = repo.create(role2).await;
    
    assert!(result.is_err());
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database operations |
| `uuid` | Role identifiers |
| `anyhow` | Error handling |
| `async-trait` | Async trait support |

### Internal Dependencies

- [auth-core/models/role.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role entity
- [auth-core/services/role_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - RoleStore trait
- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/error.md) - AuthError

---

## Related Files

- [models/role.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role model
- [services/role_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Role service

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 112  
**Security Level**: CRITICAL
