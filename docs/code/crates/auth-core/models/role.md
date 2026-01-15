# models/role.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/role.rs`  
**Crate**: `auth-core`  
**Module**: `models::role`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Defines authorization roles and permissions

## Purpose

Defines the `Role` model for Role-Based Access Control (RBAC), including role hierarchy, permissions, and constraints.

### Problem It Solves

- Implements RBAC for authorization
- Supports role hierarchy (parent-child relationships)
- Manages fine-grained permissions
- Enables multi-tenancy with tenant-specific roles

---

## Detailed Code Breakdown

### Struct: `Role`

**Purpose**: Represents an authorization role with associated permissions

**Fields**:

| Field | Type | Description | Security Notes |
|-------|------|-------------|----------------|
| `id` | `Uuid` | Unique role identifier | Primary key |
| `tenant_id` | `Uuid` | Tenant context | Multi-tenancy isolation |
| `name` | `String` | Role name | E.g., "UserManager", "Auditor" |
| `description` | `Option<String>` | Role description | Human-readable purpose |
| `parent_role_id` | `Option<Uuid>` | Parent role | Role hierarchy support |
| `is_system_role` | `bool` | System-defined role | Cannot be deleted/modified |
| `permissions` | `Json<Vec<String>>` | Permission codes | E.g., ["users:read:tenant"] |
| `constraints` | `Json<HashMap<String, String>>` | ABAC constraints | Time-based, IP-based, etc. |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Audit trail |
| `updated_at` | `Option<DateTime<Utc>>` | Last update timestamp | Audit trail |

**Derive Macros**:
- `Debug`: Debugging support
- `Clone`: Cheap cloning
- `Serialize`, `Deserialize`: JSON serialization
- `sqlx::FromRow`: Database row mapping

---

## Role Hierarchy

### Purpose

Enables role inheritance where child roles inherit parent permissions.

### Example Hierarchy

```
SuperAdmin (system role)
  ├── TenantAdmin
  │     ├── UserManager
  │     └── AuditorRole
  └── SystemOperator
```

### Implementation

```rust
pub async fn get_effective_permissions(role_id: Uuid) -> Result<Vec<String>> {
    let mut permissions = Vec::new();
    let mut current_role_id = Some(role_id);
    
    // Traverse hierarchy upward
    while let Some(id) = current_role_id {
        let role = find_role_by_id(id).await?;
        permissions.extend(role.permissions.0);
        current_role_id = role.parent_role_id;
    }
    
    // Deduplicate
    permissions.sort();
    permissions.dedup();
    Ok(permissions)
}
```

**Example**:
- `UserManager` has permissions: `["users:read:tenant", "users:write:tenant"]`
- Parent `TenantAdmin` has: `["roles:read:tenant", "audit:read:tenant"]`
- Effective permissions: All four permissions combined

---

## Permission Format

### Structure

**Format**: `resource:action:scope`

**Components**:
1. **Resource**: What is being accessed (e.g., `users`, `roles`, `audit_logs`)
2. **Action**: What operation (e.g., `read`, `write`, `delete`)
3. **Scope**: Permission scope (e.g., `global`, `tenant`, `organization`, `self`)

### Examples

| Permission | Meaning |
|------------|---------|
| `users:read:tenant` | Read users within tenant |
| `users:write:global` | Create/update users globally (SuperAdmin) |
| `users:delete:tenant` | Delete users within tenant |
| `roles:read:tenant` | Read roles within tenant |
| `roles:write:tenant` | Create/update roles within tenant |
| `audit_logs:read:organization` | Read audit logs for organization |
| `profile:write:self` | Update own profile only |

### Wildcard Support

```rust
// Check if permission matches pattern
pub fn matches_permission(required: &str, granted: &str) -> bool {
    if granted == "*:*:*" {
        return true;  // SuperAdmin wildcard
    }
    
    let req_parts: Vec<&str> = required.split(':').collect();
    let grant_parts: Vec<&str> = granted.split(':').collect();
    
    for (req, grant) in req_parts.iter().zip(grant_parts.iter()) {
        if grant != &"*" && req != grant {
            return false;
        }
    }
    
    true
}
```

**Examples**:
- `users:*:tenant` grants `users:read:tenant`, `users:write:tenant`, `users:delete:tenant`
- `*:read:tenant` grants read access to all resources in tenant

---

## System Roles

### Purpose

Pre-defined roles that cannot be modified or deleted.

### Examples

#### 1. SuperAdmin
```rust
Role {
    id: Uuid::new_v4(),
    tenant_id: Uuid::nil(),  // Global
    name: "SuperAdmin".to_string(),
    description: Some("Full system access".to_string()),
    parent_role_id: None,
    is_system_role: true,
    permissions: Json(vec!["*:*:*".to_string()]),  // Wildcard
    constraints: Json(HashMap::new()),
    created_at: Utc::now(),
    updated_at: None,
}
```

#### 2. TenantAdmin
```rust
Role {
    name: "TenantAdmin".to_string(),
    permissions: Json(vec![
        "users:*:tenant".to_string(),
        "roles:*:tenant".to_string(),
        "audit:read:tenant".to_string(),
    ]),
    is_system_role: true,
    // ...
}
```

#### 3. UserManager
```rust
Role {
    name: "UserManager".to_string(),
    permissions: Json(vec![
        "users:read:tenant".to_string(),
        "users:write:tenant".to_string(),
    ]),
    parent_role_id: Some(tenant_admin_id),
    is_system_role: true,
    // ...
}
```

---

## Constraints (ABAC)

### Purpose

Attribute-Based Access Control for fine-grained authorization.

### Constraint Types

#### 1. Time-Based
```rust
constraints: Json(hashmap!{
    "time_start" => "09:00",
    "time_end" => "17:00",
    "timezone" => "UTC",
})
```

**Validation**:
```rust
if let Some(start) = constraints.get("time_start") {
    let current_hour = Utc::now().hour();
    let start_hour = start.parse::<u32>().unwrap();
    if current_hour < start_hour {
        return Err(AuthError::AuthorizationDenied { ... });
    }
}
```

#### 2. IP-Based
```rust
constraints: Json(hashmap!{
    "allowed_ips" => "10.0.0.0/8,192.168.1.0/24",
})
```

**Validation**:
```rust
if let Some(allowed_ips) = constraints.get("allowed_ips") {
    let ip_ranges: Vec<&str> = allowed_ips.split(',').collect();
    if !ip_ranges.iter().any(|range| ip_in_range(&request.ip, range)) {
        return Err(AuthError::AuthorizationDenied { ... });
    }
}
```

#### 3. Data Ownership
```rust
constraints: Json(hashmap!{
    "ownership" => "self_only",
})
```

**Validation**:
```rust
if constraints.get("ownership") == Some(&"self_only".to_string()) {
    if resource.owner_id != user.id {
        return Err(AuthError::AuthorizationDenied { ... });
    }
}
```

---

## Struct: `CreateRoleRequest`

**Purpose**: DTO for creating new roles

**Fields**:
- `name`: Role name (unique within tenant)
- `description`: Optional description
- `parent_role_id`: Optional parent for hierarchy
- `permissions`: List of permission codes
- `constraints`: Optional ABAC constraints

**Validation**:
```rust
// 1. Name uniqueness
if role_exists(&request.name, tenant_id).await? {
    return Err(AuthError::Conflict { message: "Role name exists" });
}

// 2. Parent role exists
if let Some(parent_id) = request.parent_role_id {
    if find_role_by_id(parent_id).await?.is_none() {
        return Err(AuthError::ValidationError { message: "Parent role not found" });
    }
}

// 3. Permission format
for perm in &request.permissions {
    if !is_valid_permission_format(perm) {
        return Err(AuthError::ValidationError { 
            message: format!("Invalid permission format: {}", perm) 
        });
    }
}
```

---

## Struct: `UpdateRoleRequest`

**Purpose**: DTO for updating existing roles

**Fields**: All optional (partial update)
- `name`: New role name
- `description`: New description
- `parent_role_id`: New parent role
- `permissions`: New permission list
- `constraints`: New constraints

**Security**:
```rust
// Cannot update system roles
if role.is_system_role {
    return Err(AuthError::AuthorizationDenied {
        permission: "roles:write".to_string(),
        resource: "system_role".to_string(),
    });
}
```

---

## Permission Checking

### Service Method

```rust
pub async fn check_permission(
    user_id: Uuid,
    required_permission: &str,
) -> Result<bool> {
    // 1. Get user's roles
    let user_roles = get_user_roles(user_id).await?;
    
    // 2. Get effective permissions (including inherited)
    let mut all_permissions = Vec::new();
    for role in user_roles {
        let perms = get_effective_permissions(role.id).await?;
        all_permissions.extend(perms);
    }
    
    // 3. Check if any permission matches
    for granted in &all_permissions {
        if matches_permission(required_permission, granted) {
            // 4. Check constraints
            if check_constraints(&role.constraints).await? {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
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
    constraints JSON NOT NULL DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NULL,
    
    UNIQUE KEY unique_role_name (tenant_id, name),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_parent_role_id (parent_role_id),
    
    FOREIGN KEY (parent_role_id) REFERENCES roles(id) ON DELETE SET NULL
);

CREATE TABLE user_roles (
    user_id CHAR(36) NOT NULL,
    role_id CHAR(36) NOT NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    assigned_by CHAR(36),
    
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);
```

---

## Security Considerations

### Role Assignment

**Authorization Required**: Only users with `roles:write:tenant` can assign roles

```rust
// Check if assigner has permission
if !check_permission(assigner_id, "roles:write:tenant").await? {
    return Err(AuthError::AuthorizationDenied { ... });
}

// Cannot assign higher-privilege role
if is_higher_privilege(&target_role, &assigner_roles).await? {
    return Err(AuthError::AuthorizationDenied { 
        permission: "roles:write".to_string(),
        resource: "higher_privilege_role".to_string(),
    });
}
```

### System Role Protection

```rust
// Cannot delete system roles
if role.is_system_role {
    return Err(AuthError::AuthorizationDenied { ... });
}

// Cannot modify system role permissions
if role.is_system_role && request.permissions.is_some() {
    return Err(AuthError::AuthorizationDenied { ... });
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `serde` | JSON serialization |
| `uuid` | Unique identifiers |
| `chrono` | Timestamps |
| `sqlx` | Database mapping |

### Internal Dependencies

- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs) - Permission checking

---

## Related Files

- [services/role_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/role_service.rs) - Role operations
- [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs) - Authorization logic
- [repositories/role_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/role_repository.rs) - Role persistence

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 38  
**Security Level**: CRITICAL
