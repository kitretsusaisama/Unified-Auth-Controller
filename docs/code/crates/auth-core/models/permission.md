# models/permission.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/permission.rs`  
**Crate**: `auth-core`  
**Module**: `models::permission`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Authorization foundation

## Purpose

Defines the `Permission` model for fine-grained access control, representing individual permissions that can be granted to roles.

### Problem It Solves

- Enables fine-grained authorization (beyond role-based)
- Defines atomic permissions (read, write, delete on specific resources)
- Supports conditional permissions (ABAC - Attribute-Based Access Control)
- Provides permission-role mapping for RBAC

---

## Detailed Code Breakdown

### Struct: `Permission`

**Purpose**: Represents a single permission in the system

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `id` | `Uuid` | Unique permission identifier | Primary key |
| `code` | `String` | Permission code (e.g., `users:read:tenant`) | 1-100 characters, unique |
| `name` | `String` | Human-readable name | 1-255 characters |
| `description` | `Option<String>` | Permission description | Optional |
| `resource_type` | `Option<String>` | Resource being accessed | E.g., "users", "roles" |
| `action` | `Option<String>` | Action being performed | E.g., "read", "write", "delete" |
| `conditions` | `serde_json::Value` | ABAC conditions | JSON object |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Audit trail |

**Permission Code Format**: `resource:action:scope`

**Examples**:
- `users:read:tenant` - Read users within tenant
- `users:write:global` - Create/update users globally
- `roles:delete:tenant` - Delete roles within tenant
- `audit_logs:read:organization` - Read audit logs for organization

---

### Permission Components

#### 1. Resource Type

**Purpose**: What is being accessed

**Examples**:
- `users` - User entities
- `roles` - Role entities
- `permissions` - Permission entities
- `tenants` - Tenant entities
- `organizations` - Organization entities
- `sessions` - Session entities
- `audit_logs` - Audit log entries
- `subscriptions` - Subscription plans

#### 2. Action

**Purpose**: What operation is being performed

**Standard Actions**:
- `read` - View/list resources
- `write` - Create/update resources
- `delete` - Delete resources
- `execute` - Execute operations (e.g., password reset)
- `manage` - Full control (read + write + delete)

**Custom Actions**:
- `approve` - Approve requests
- `export` - Export data
- `import` - Import data
- `configure` - Change settings

#### 3. Scope

**Purpose**: Permission boundary

**Scopes**:
- `self` - Own resources only
- `tenant` - Within tenant
- `organization` - Within organization
- `global` - Across all tenants (SuperAdmin)

---

### Conditions (ABAC)

**Purpose**: Conditional permissions based on attributes

**Example Conditions**:

```json
{
  "time_restriction": {
    "start_hour": 9,
    "end_hour": 17,
    "timezone": "UTC"
  },
  "ip_restriction": {
    "allowed_ranges": ["10.0.0.0/8", "192.168.1.0/24"]
  },
  "ownership": {
    "require_owner": true
  },
  "mfa_required": true
}
```

**Validation Logic**:

```rust
pub fn check_conditions(
    permission: &Permission,
    context: &AccessContext,
) -> Result<bool> {
    let conditions = permission.conditions.as_object().ok_or(...)?;
    
    // Time restriction
    if let Some(time) = conditions.get("time_restriction") {
        let current_hour = Utc::now().hour();
        let start = time["start_hour"].as_u64().unwrap();
        let end = time["end_hour"].as_u64().unwrap();
        
        if current_hour < start as u32 || current_hour >= end as u32 {
            return Ok(false);
        }
    }
    
    // IP restriction
    if let Some(ip_config) = conditions.get("ip_restriction") {
        let allowed_ranges = ip_config["allowed_ranges"]
            .as_array().ok_or(...)?;
        
        if !is_ip_in_ranges(&context.ip_address, allowed_ranges) {
            return Ok(false);
        }
    }
    
    // Ownership check
    if let Some(ownership) = conditions.get("ownership") {
        if ownership["require_owner"].as_bool().unwrap_or(false) {
            if context.user_id != context.resource_owner_id {
                return Ok(false);
            }
        }
    }
    
    // MFA requirement
    if conditions.get("mfa_required").and_then(|v| v.as_bool()).unwrap_or(false) {
        if !context.mfa_verified {
            return Ok(false);
        }
    }
    
    Ok(true)
}
```

---

### Struct: `RolePermission`

**Purpose**: Maps permissions to roles (many-to-many relationship)

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `role_id` | `Uuid` | Role identifier |
| `permission_id` | `Uuid` | Permission identifier |
| `granted` | `bool` | Permission granted (true) or denied (false) |
| `conditions` | `serde_json::Value` | Additional conditions for this role-permission |
| `created_at` | `DateTime<Utc>` | When permission was granted |

**Purpose of `granted` Field**:
- `true`: Permission explicitly granted
- `false`: Permission explicitly denied (overrides inherited permissions)

**Example**:
```rust
// Grant permission
RolePermission {
    role_id: user_manager_role_id,
    permission_id: users_read_permission_id,
    granted: true,
    conditions: json!({}),
    created_at: Utc::now(),
}

// Deny permission (override)
RolePermission {
    role_id: restricted_manager_role_id,
    permission_id: users_delete_permission_id,
    granted: false,  // Explicitly denied
    conditions: json!({}),
    created_at: Utc::now(),
}
```

---

## Permission Checking Algorithm

### Step-by-Step Process

```rust
pub async fn has_permission(
    user_id: Uuid,
    required_permission: &str,
    context: &AccessContext,
) -> Result<bool> {
    // 1. Get user's roles
    let user_roles = get_user_roles(user_id).await?;
    
    // 2. Get all role-permission mappings
    let mut role_permissions = Vec::new();
    for role in &user_roles {
        let perms = get_role_permissions(role.id).await?;
        role_permissions.extend(perms);
    }
    
    // 3. Check for explicit denials first
    for rp in &role_permissions {
        if !rp.granted && matches_permission_code(&rp.permission.code, required_permission) {
            return Ok(false);  // Explicit denial
        }
    }
    
    // 4. Check for grants
    for rp in &role_permissions {
        if rp.granted && matches_permission_code(&rp.permission.code, required_permission) {
            // 5. Check conditions
            if check_conditions(&rp.permission, context)? {
                return Ok(true);  // Permission granted and conditions met
            }
        }
    }
    
    Ok(false)  // No matching permission found
}
```

---

## Permission Wildcards

### Wildcard Support

**Format**: Use `*` for wildcard matching

**Examples**:

| Permission Code | Matches |
|-----------------|---------|
| `users:*:tenant` | `users:read:tenant`, `users:write:tenant`, `users:delete:tenant` |
| `*:read:tenant` | `users:read:tenant`, `roles:read:tenant`, `sessions:read:tenant` |
| `users:read:*` | `users:read:tenant`, `users:read:organization`, `users:read:global` |
| `*:*:*` | All permissions (SuperAdmin) |

**Matching Logic**:

```rust
pub fn matches_permission_code(granted: &str, required: &str) -> bool {
    let granted_parts: Vec<&str> = granted.split(':').collect();
    let required_parts: Vec<&str> = required.split(':').collect();
    
    if granted_parts.len() != 3 || required_parts.len() != 3 {
        return false;
    }
    
    for (g, r) in granted_parts.iter().zip(required_parts.iter()) {
        if g != &"*" && g != r {
            return false;
        }
    }
    
    true
}
```

---

## Database Schema

```sql
CREATE TABLE permissions (
    id CHAR(36) PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(50),
    action VARCHAR(50),
    conditions JSON DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_code (code),
    INDEX idx_resource_type (resource_type),
    INDEX idx_action (action)
);

CREATE TABLE role_permissions (
    role_id CHAR(36) NOT NULL,
    permission_id CHAR(36) NOT NULL,
    granted BOOLEAN NOT NULL DEFAULT TRUE,
    conditions JSON DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);
```

---

## Predefined Permissions

### System Permissions

```rust
pub fn create_system_permissions() -> Vec<Permission> {
    vec![
        // User permissions
        Permission {
            id: Uuid::new_v4(),
            code: "users:read:tenant".to_string(),
            name: "Read Users (Tenant)".to_string(),
            description: Some("View users within tenant".to_string()),
            resource_type: Some("users".to_string()),
            action: Some("read".to_string()),
            conditions: json!({}),
            created_at: Utc::now(),
        },
        Permission {
            code: "users:write:tenant".to_string(),
            name: "Write Users (Tenant)".to_string(),
            // ...
        },
        Permission {
            code: "users:delete:tenant".to_string(),
            name: "Delete Users (Tenant)".to_string(),
            // ...
        },
        
        // Role permissions
        Permission {
            code: "roles:read:tenant".to_string(),
            name: "Read Roles (Tenant)".to_string(),
            // ...
        },
        
        // Audit permissions
        Permission {
            code: "audit_logs:read:tenant".to_string(),
            name: "Read Audit Logs (Tenant)".to_string(),
            // ...
        },
        
        // SuperAdmin wildcard
        Permission {
            code: "*:*:*".to_string(),
            name: "SuperAdmin (All Permissions)".to_string(),
            // ...
        },
    ]
}
```

---

## Security Considerations

### Permission Granularity

**Too Coarse**: `admin` (all permissions)
- Hard to audit
- Violates least privilege

**Too Fine**: `users:read:tenant:active:email_verified`
- Too many permissions
- Hard to manage

**Balanced**: `users:read:tenant` + conditions
- Manageable number of permissions
- Flexible with conditions

### Explicit Denials

**Purpose**: Override inherited permissions

**Example**:
```rust
// UserManager role inherits from TenantAdmin
// TenantAdmin has users:delete:tenant
// But UserManager explicitly denies it

RolePermission {
    role_id: user_manager_id,
    permission_id: users_delete_id,
    granted: false,  // Explicit denial overrides inheritance
}
```

### Condition Validation

**Security**: Always validate conditions server-side

**Bad** (client-side only):
```javascript
// DON'T: Client can bypass
if (currentHour >= 9 && currentHour <= 17) {
    allowAccess();
}
```

**Good** (server-side):
```rust
// DO: Server validates
if !check_conditions(&permission, &context)? {
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
| `validator` | Input validation |

### Internal Dependencies

- [models/role.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/role.rs) - Role entity
- [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs) - Permission checking

---

## Related Files

- [models/role.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/role.md) - Role model
- [services/authorization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/authorization.rs) - Authorization service
- [repositories/permission_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories) - Permission persistence

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 29  
**Security Level**: CRITICAL
