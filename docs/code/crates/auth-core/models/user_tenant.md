# models/user_tenant.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/user_tenant.rs`  
**Crate**: `auth-core`  
**Module**: `models::user_tenant`  
**Layer**: Domain (Data Model)  
**Security-Critical**: ⚠️ **MEDIUM** - Multi-tenancy relationships

## Purpose

Defines the many-to-many relationship between users and tenants, enabling users to belong to multiple tenants with different statuses and roles.

### Problem It Solves

- Multi-tenant user access
- User status per tenant
- Cross-tenant user management
- Role assignments per tenant
- Tenant switching for users

---

## Detailed Code Breakdown

### Struct: `UserTenant`

**Purpose**: User-tenant relationship

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Uuid` | User identifier |
| `tenant_id` | `Uuid` | Tenant identifier |
| `status` | `UserTenantStatus` | Relationship status |
| `joined_at` | `DateTime<Utc>` | When user joined tenant |
| `last_accessed_at` | `Option<DateTime<Utc>>` | Last tenant access |

**Example**:
```rust
let user_tenant = UserTenant {
    user_id: Uuid::new_v4(),
    tenant_id: Uuid::new_v4(),
    status: UserTenantStatus::Active,
    joined_at: Utc::now(),
    last_accessed_at: Some(Utc::now()),
};
```

---

### Enum: `UserTenantStatus`

**Purpose**: Status of user within tenant

**Variants**:

| Variant | Description |
|---------|-------------|
| `Active` | User has full access |
| `Suspended` | User temporarily blocked |
| `Pending` | Invitation not accepted |

**Default**: `Pending`

---

### Struct: `CreateUserTenantRequest`

**Purpose**: Request to add user to tenant

**Fields**:
- `user_id`: User to add
- `tenant_id`: Target tenant
- `status`: Optional initial status (defaults to Pending)

---

### Struct: `UserRole`

**Purpose**: Role assignment within tenant

**Fields**:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Assignment identifier |
| `user_id` | `Uuid` | User |
| `tenant_id` | `Uuid` | Tenant context |
| `role_id` | `Uuid` | Assigned role |
| `granted_by` | `Uuid` | Who granted role |
| `granted_at` | `DateTime<Utc>` | When granted |
| `expires_at` | `Option<DateTime<Utc>>` | Optional expiration |
| `revoked_at` | `Option<DateTime<Utc>>` | Revocation time |
| `revoked_by` | `Option<Uuid>` | Who revoked |

---

## Multi-Tenancy Patterns

### Pattern 1: User Invitation

```rust
// 1. Create pending user-tenant relationship
let user_tenant = UserTenant {
    user_id,
    tenant_id,
    status: UserTenantStatus::Pending,
    joined_at: Utc::now(),
    last_accessed_at: None,
};

// 2. Send invitation email
send_invitation_email(user_id, tenant_id).await?;

// 3. User accepts invitation
sqlx::query!(
    "UPDATE user_tenants SET status = 'Active', last_accessed_at = ? WHERE user_id = ? AND tenant_id = ?",
    Utc::now(),
    user_id.to_string(),
    tenant_id.to_string()
).execute(&pool).await?;
```

---

### Pattern 2: Tenant Switching

```rust
pub async fn switch_tenant(
    user_id: Uuid,
    new_tenant_id: Uuid,
    pool: &MySqlPool,
) -> Result<()> {
    // Verify user has access to tenant
    let user_tenant = sqlx::query_as!(
        UserTenant,
        "SELECT * FROM user_tenants WHERE user_id = ? AND tenant_id = ?",
        user_id.to_string(),
        new_tenant_id.to_string()
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::Unauthorized {
        message: "User not member of tenant".to_string(),
    })?;
    
    // Check status
    if !matches!(user_tenant.status, UserTenantStatus::Active) {
        return Err(AuthError::Unauthorized {
            message: "User suspended in tenant".to_string(),
        });
    }
    
    // Update last accessed
    sqlx::query!(
        "UPDATE user_tenants SET last_accessed_at = ? WHERE user_id = ? AND tenant_id = ?",
        Utc::now(),
        user_id.to_string(),
        new_tenant_id.to_string()
    ).execute(pool).await?;
    
    Ok(())
}
```

---

### Pattern 3: Cross-Tenant User Management

```rust
pub async fn get_user_tenants(
    user_id: Uuid,
    pool: &MySqlPool,
) -> Result<Vec<UserTenant>> {
    let tenants = sqlx::query_as!(
        UserTenant,
        "SELECT * FROM user_tenants WHERE user_id = ? AND status = 'Active'",
        user_id.to_string()
    )
    .fetch_all(pool)
    .await?;
    
    Ok(tenants)
}
```

---

## Role Assignment

### Assign Role to User in Tenant

```rust
pub async fn assign_role(
    user_id: Uuid,
    tenant_id: Uuid,
    role_id: Uuid,
    granted_by: Uuid,
    expires_at: Option<DateTime<Utc>>,
    pool: &MySqlPool,
) -> Result<UserRole> {
    let user_role = UserRole {
        id: Uuid::new_v4(),
        user_id,
        tenant_id,
        role_id,
        granted_by,
        granted_at: Utc::now(),
        expires_at,
        revoked_at: None,
        revoked_by: None,
    };
    
    sqlx::query!(
        "INSERT INTO user_roles (id, user_id, tenant_id, role_id, granted_by, granted_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        user_role.id.to_string(),
        user_id.to_string(),
        tenant_id.to_string(),
        role_id.to_string(),
        granted_by.to_string(),
        user_role.granted_at,
        expires_at
    ).execute(pool).await?;
    
    Ok(user_role)
}
```

---

### Revoke Role

```rust
pub async fn revoke_role(
    user_role_id: Uuid,
    revoked_by: Uuid,
    pool: &MySqlPool,
) -> Result<()> {
    sqlx::query!(
        "UPDATE user_roles SET revoked_at = ?, revoked_by = ? WHERE id = ?",
        Utc::now(),
        revoked_by.to_string(),
        user_role_id.to_string()
    ).execute(pool).await?;
    
    Ok(())
}
```

---

## Database Schema

```sql
CREATE TABLE user_tenants (
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed_at TIMESTAMP NULL,
    
    PRIMARY KEY (user_id, tenant_id),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_status (status),
    INDEX idx_last_accessed (last_accessed_at),
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE TABLE user_roles (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    role_id CHAR(36) NOT NULL,
    granted_by CHAR(36) NOT NULL,
    granted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL,
    revoked_at TIMESTAMP NULL,
    revoked_by CHAR(36),
    
    INDEX idx_user_tenant (user_id, tenant_id),
    INDEX idx_role_id (role_id),
    INDEX idx_expires_at (expires_at),
    INDEX idx_revoked_at (revoked_at),
    
    FOREIGN KEY (user_id, tenant_id) REFERENCES user_tenants(user_id, tenant_id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (granted_by) REFERENCES users(id),
    FOREIGN KEY (revoked_by) REFERENCES users(id)
);
```

---

## Usage Examples

### Example 1: User Joins Multiple Tenants

```rust
// User joins Company A
let company_a = UserTenant {
    user_id,
    tenant_id: company_a_id,
    status: UserTenantStatus::Active,
    joined_at: Utc::now(),
    last_accessed_at: Some(Utc::now()),
};

// User joins Company B
let company_b = UserTenant {
    user_id,
    tenant_id: company_b_id,
    status: UserTenantStatus::Active,
    joined_at: Utc::now(),
    last_accessed_at: None,
};

// User has different roles in each tenant
assign_role(user_id, company_a_id, admin_role_id, system_id, None, &pool).await?;
assign_role(user_id, company_b_id, viewer_role_id, system_id, None, &pool).await?;
```

---

### Example 2: Temporary Role Assignment

```rust
// Grant temporary admin access for 24 hours
let expires_at = Utc::now() + Duration::hours(24);

assign_role(
    user_id,
    tenant_id,
    admin_role_id,
    granting_admin_id,
    Some(expires_at),
    &pool
).await?;
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `serde` | Serialization |
| `uuid` | Identifiers |
| `chrono` | Timestamps |
| `validator` | Validation |

---

## Related Files

- [models/user.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/user.md) - User model
- [models/tenant.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/tenant.md) - Tenant model
- [models/role.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/role.md) - Role model

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 48  
**Security Level**: MEDIUM
