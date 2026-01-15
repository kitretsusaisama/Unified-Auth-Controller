# models/organization.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/organization.rs`  
**Crate**: `auth-core`  
**Module**: `models::organization`  
**Layer**: Domain  
**Security-Critical**: ⚠️ **MEDIUM** - Organizational hierarchy

## Purpose

Defines the `Organization` model representing the top-level entity in the multi-tenancy hierarchy. Organizations contain multiple tenants.

### Problem It Solves

- Enables enterprise customers with multiple tenants (departments, subsidiaries)
- Provides organizational hierarchy (Organization → Tenants → Users)
- Manages organization-level settings and billing
- Supports B2B SaaS model

---

## Detailed Code Breakdown

### Struct: `Organization`

**Purpose**: Represents a customer organization (enterprise account)

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `id` | `Uuid` | Unique organization identifier | Primary key |
| `name` | `String` | Organization name | 1-255 characters |
| `domain` | `Option<String>` | Organization domain | Valid URL |
| `status` | `OrganizationStatus` | Organization status | Active, Suspended, Deleted |
| `settings` | `serde_json::Value` | Organization settings | JSON object |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Audit trail |
| `updated_at` | `DateTime<Utc>` | Last update timestamp | Audit trail |

**Hierarchy**: `Organization` (1) → `Tenant` (many) → `User` (many)

---

### Enum: `OrganizationStatus`

**Purpose**: Organization lifecycle states

**Variants**:

1. **`Active`**: Organization is operational
   - All tenants can operate
   - Billing active

2. **`Suspended`**: Organization temporarily disabled
   - All tenants suspended
   - Users cannot authenticate
   - Reason: Payment failure, contract violation

3. **`Deleted`**: Organization soft-deleted
   - All tenants marked deleted
   - Data retained for compliance
   - Can be restored within retention period

**Default**: `Active`

**Cascade Effect**: Organization status affects all child tenants

---

### Settings Object

**Purpose**: Organization-wide configuration

**Example**:
```json
{
  "billing": {
    "plan": "enterprise",
    "billing_email": "billing@acme.com",
    "payment_method": "invoice"
  },
  "security": {
    "enforce_mfa": true,
    "allowed_auth_methods": ["password", "saml", "oidc"],
    "session_timeout_minutes": 30,
    "ip_whitelist": ["203.0.113.0/24"]
  },
  "compliance": {
    "data_residency": "EU",
    "retention_days": 2555,
    "gdpr_enabled": true,
    "soc2_compliant": true
  },
  "branding": {
    "logo_url": "https://cdn.acme.com/logo.png",
    "primary_color": "#1E40AF",
    "support_email": "support@acme.com"
  }
}
```

---

### Method: `Organization::is_active()`

**Signature**: `pub fn is_active(&self) -> bool`

**Purpose**: Check if organization is operational

**Returns**: `true` if status is `Active`

**Usage**:
```rust
if !organization.is_active() {
    return Err(AuthError::Unauthorized {
        message: "Organization is not active".to_string()
    });
}
```

---

### Method: `Organization::get_domain()`

**Signature**: `pub fn get_domain(&self) -> Option<&str>`

**Purpose**: Get organization's domain

**Returns**: Domain if configured

**Usage**:
```rust
let domain = organization.get_domain().unwrap_or("default.com");
```

---

### Method: `Organization::has_custom_settings()`

**Signature**: `pub fn has_custom_settings(&self) -> bool`

**Purpose**: Check if organization has custom settings

**Logic**:
```rust
!self.settings.is_null() && 
self.settings.as_object().map_or(false, |obj| !obj.is_empty())
```

---

### Struct: `CreateOrganizationRequest`

**Purpose**: DTO for creating new organization

**Fields**:
- `name`: Organization name (required)
- `domain`: Organization domain (optional)
- `settings`: Initial settings (optional)

**Validation**:
```rust
// 1. Name uniqueness
if organization_exists_by_name(&request.name).await? {
    return Err(AuthError::Conflict {
        message: "Organization name already exists".to_string()
    });
}

// 2. Domain format
if let Some(ref domain) = request.domain {
    if !is_valid_domain(domain) {
        return Err(AuthError::ValidationError {
            message: "Invalid domain format".to_string()
        });
    }
}
```

---

### Struct: `UpdateOrganizationRequest`

**Purpose**: DTO for updating existing organization

**Fields**: All optional (partial update)
- `id`: Organization to update
- `name`: New name
- `domain`: New domain
- `settings`: Updated settings
- `status`: New status

**Security**:
```rust
// Only SuperAdmin can change organization status
if request.status.is_some() && !user.is_super_admin() {
    return Err(AuthError::AuthorizationDenied {
        permission: "organizations:manage:global".to_string(),
        resource: "organization_status".to_string(),
    });
}
```

---

## Organizational Hierarchy

### Structure

```
Organization (Acme Corporation)
├── Tenant (Engineering Department)
│   ├── User (Alice - Engineer)
│   ├── User (Bob - Manager)
│   └── User (Charlie - Engineer)
├── Tenant (Sales Department)
│   ├── User (David - Sales Rep)
│   └── User (Eve - Sales Manager)
└── Tenant (Finance Department)
    ├── User (Frank - Accountant)
    └── User (Grace - CFO)
```

### Benefits

1. **Centralized Billing**: One invoice for entire organization
2. **Unified Settings**: Security policies apply to all tenants
3. **Cross-Tenant Reporting**: Organization-wide analytics
4. **Simplified Management**: Manage multiple departments from one account

---

## Use Cases

### Enterprise Customer

**Scenario**: Large company with multiple departments

**Structure**:
- Organization: "Acme Corporation"
- Tenants: "Engineering", "Sales", "Finance", "HR"
- Users: Employees in each department

**Benefits**:
- Department isolation (data security)
- Centralized billing and management
- Organization-wide policies

---

### Multi-Brand Company

**Scenario**: Holding company with multiple brands

**Structure**:
- Organization: "Global Holdings Inc"
- Tenants: "Brand A", "Brand B", "Brand C"
- Users: Employees of each brand

**Benefits**:
- Brand isolation
- Shared infrastructure
- Consolidated reporting

---

### Reseller/Agency

**Scenario**: Agency managing multiple client accounts

**Structure**:
- Organization: "Digital Agency"
- Tenants: "Client A", "Client B", "Client C"
- Users: Client employees + agency staff

**Benefits**:
- Client isolation
- Agency-wide management
- Per-client billing

---

## Database Schema

```sql
CREATE TABLE organizations (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    domain VARCHAR(255) UNIQUE,
    status VARCHAR(20) DEFAULT 'Active',
    settings JSON DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_name (name),
    INDEX idx_domain (domain),
    INDEX idx_status (status)
);

-- Tenants belong to organizations
ALTER TABLE tenants
    ADD COLUMN organization_id CHAR(36) NOT NULL,
    ADD FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE;
```

---

## Cascade Operations

### Organization Suspension

**Effect**: Suspends all child tenants

```rust
pub async fn suspend_organization(org_id: Uuid) -> Result<()> {
    // 1. Suspend organization
    update_organization_status(org_id, OrganizationStatus::Suspended).await?;
    
    // 2. Suspend all tenants
    let tenants = get_organization_tenants(org_id).await?;
    for tenant in tenants {
        suspend_tenant(tenant.id).await?;
    }
    
    // 3. Revoke all active sessions
    revoke_organization_sessions(org_id).await?;
    
    Ok(())
}
```

### Organization Deletion

**Effect**: Soft-deletes organization and all tenants

```rust
pub async fn delete_organization(org_id: Uuid) -> Result<()> {
    // 1. Mark organization as deleted
    update_organization_status(org_id, OrganizationStatus::Deleted).await?;
    
    // 2. Mark all tenants as deleted
    let tenants = get_organization_tenants(org_id).await?;
    for tenant in tenants {
        delete_tenant(tenant.id).await?;
    }
    
    // 3. Preserve data for compliance (soft delete)
    // Data remains in database with deleted_at timestamp
    
    Ok(())
}
```

---

## Security Considerations

### Organization Isolation

**Critical**: Prevent cross-organization data leakage

**Enforcement**:
```rust
// Always filter by organization
let tenants = sqlx::query_as!(
    Tenant,
    "SELECT * FROM tenants WHERE organization_id = ?",
    org_id
).fetch_all(&pool).await?;

// Validate user belongs to organization
pub async fn validate_organization_access(
    user_id: Uuid,
    org_id: Uuid,
) -> Result<bool> {
    let user_orgs = get_user_organizations(user_id).await?;
    Ok(user_orgs.iter().any(|org| org.id == org_id))
}
```

### Settings Inheritance

**Pattern**: Organization settings override tenant settings

```rust
pub fn get_effective_settings(
    tenant: &Tenant,
    organization: &Organization,
) -> serde_json::Value {
    let mut settings = organization.settings.clone();
    
    // Merge tenant settings (tenant overrides organization)
    if let Some(tenant_settings) = tenant.auth_config.as_object() {
        if let Some(org_settings) = settings.as_object_mut() {
            for (key, value) in tenant_settings {
                org_settings.insert(key.clone(), value.clone());
            }
        }
    }
    
    settings
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

- [models/tenant.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/tenant.rs) - Child tenants
- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - Organization users

---

## Related Files

- [models/tenant.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-core/models/tenant.md) - Tenant model
- [repositories/organization_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories) - Organization persistence
- [services/organization_service.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services) - Organization operations

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 69  
**Security Level**: MEDIUM
