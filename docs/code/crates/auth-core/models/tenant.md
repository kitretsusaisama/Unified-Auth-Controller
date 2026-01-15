# models/tenant.rs

## File Metadata

**File Path**: `crates/auth-core/src/models/tenant.rs`  
**Crate**: `auth-core`  
**Module**: `models::tenant`  
**Layer**: Domain  
**Security-Critical**: ✅ **YES** - Multi-tenancy isolation foundation

## Purpose

Defines the `Tenant` model for multi-tenancy support, enabling isolated environments for different organizations within the same platform instance.

### Problem It Solves

- Enables SaaS multi-tenancy (multiple customers on single platform)
- Provides data isolation between tenants
- Supports custom branding and configuration per tenant
- Manages tenant lifecycle (active, suspended, deleted)

---

## Detailed Code Breakdown

### Struct: `Tenant`

**Purpose**: Represents an isolated tenant environment

**Fields**:

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `id` | `Uuid` | Unique tenant identifier | Primary key |
| `organization_id` | `Uuid` | Parent organization | Foreign key |
| `name` | `String` | Tenant display name | 1-255 characters |
| `slug` | `String` | URL-safe identifier | 1-100 characters, alphanumeric + hyphens |
| `custom_domain` | `Option<String>` | Custom domain (e.g., `acme.auth.com`) | Valid URL |
| `branding_config` | `serde_json::Value` | Logo, colors, theme | JSON object |
| `auth_config` | `serde_json::Value` | Authentication settings | JSON object |
| `compliance_config` | `serde_json::Value` | Compliance requirements | JSON object |
| `status` | `TenantStatus` | Tenant status | Active, Suspended, Deleted |
| `created_at` | `DateTime<Utc>` | Creation timestamp | Audit trail |
| `updated_at` | `DateTime<Utc>` | Last update timestamp | Audit trail |

**Multi-Tenancy Pattern**: Each user, role, session belongs to exactly one tenant

---

### Enum: `TenantStatus`

**Purpose**: Tenant lifecycle states

**Variants**:

1. **`Active`**: Tenant is operational
   - Users can authenticate
   - All features available

2. **`Suspended`**: Tenant temporarily disabled
   - Users cannot authenticate
   - Data preserved
   - Reasons: Payment failure, policy violation

3. **`Deleted`**: Tenant soft-deleted
   - Users cannot authenticate
   - Data retained for compliance
   - Can be restored within retention period

**Default**: `Active`

---

### Configuration Objects

#### Branding Config

**Purpose**: Customize UI appearance per tenant

**Example**:
```json
{
  "logo_url": "https://cdn.example.com/acme-logo.png",
  "primary_color": "#1E40AF",
  "secondary_color": "#10B981",
  "theme": "light",
  "company_name": "Acme Corporation"
}
```

#### Auth Config

**Purpose**: Tenant-specific authentication settings

**Example**:
```json
{
  "password_policy": {
    "min_length": 12,
    "require_uppercase": true,
    "require_numbers": true,
    "require_special": true
  },
  "mfa_required": true,
  "session_timeout_minutes": 30,
  "allowed_auth_methods": ["password", "google_oauth", "saml"]
}
```

#### Compliance Config

**Purpose**: Regulatory compliance requirements

**Example**:
```json
{
  "data_residency": "EU",
  "retention_days": 2555,
  "gdpr_enabled": true,
  "hipaa_enabled": false,
  "audit_log_required": true
}
```

---

### Method: `Tenant::is_active()`

**Signature**: `pub fn is_active(&self) -> bool`

**Purpose**: Check if tenant is operational

**Returns**: `true` if status is `Active`

**Usage**:
```rust
if !tenant.is_active() {
    return Err(AuthError::TenantNotFound { 
        tenant_id: tenant.id.to_string() 
    });
}
```

---

### Method: `Tenant::get_domain()`

**Signature**: `pub fn get_domain(&self) -> Option<&str>`

**Purpose**: Get tenant's custom domain

**Returns**: Custom domain if configured, otherwise `None`

**Usage**:
```rust
let domain = tenant.get_domain().unwrap_or("default.auth.com");
```

---

### Method: `Tenant::has_custom_branding()`

**Signature**: `pub fn has_custom_branding(&self) -> bool`

**Purpose**: Check if tenant has custom branding configured

**Logic**:
```rust
!self.branding_config.is_null() && 
self.branding_config.as_object().map_or(false, |obj| !obj.is_empty())
```

**Returns**: `true` if branding_config is non-empty object

---

### Method: `Tenant::is_valid_slug()`

**Signature**: `pub fn is_valid_slug(slug: &str) -> bool`

**Purpose**: Validate slug format

**Rules**:
- Alphanumeric characters and hyphens only
- Cannot start with hyphen
- Cannot end with hyphen

**Examples**:
- ✅ Valid: `acme-corp`, `tenant123`, `my-tenant`
- ❌ Invalid: `-acme`, `acme-`, `acme corp`, `acme_corp`

**Implementation**:
```rust
slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    && !slug.starts_with('-')
    && !slug.ends_with('-')
```

---

### Struct: `CreateTenantRequest`

**Purpose**: DTO for creating new tenant

**Fields**:
- `organization_id`: Parent organization
- `name`: Tenant display name
- `slug`: URL-safe identifier (unique)
- `custom_domain`: Optional custom domain
- `branding_config`: Optional branding settings
- `auth_config`: Optional auth settings
- `compliance_config`: Optional compliance settings

**Validation**:
```rust
// 1. Slug uniqueness
if tenant_exists_by_slug(&request.slug).await? {
    return Err(AuthError::Conflict { 
        message: "Slug already exists".to_string() 
    });
}

// 2. Slug format
if !Tenant::is_valid_slug(&request.slug) {
    return Err(AuthError::ValidationError { 
        message: "Invalid slug format".to_string() 
    });
}

// 3. Organization exists
if find_organization(request.organization_id).await?.is_none() {
    return Err(AuthError::ValidationError { 
        message: "Organization not found".to_string() 
    });
}
```

---

### Struct: `UpdateTenantRequest`

**Purpose**: DTO for updating existing tenant

**Fields**: All optional (partial update)
- `id`: Tenant to update
- `name`: New name
- `custom_domain`: New domain
- `branding_config`: Updated branding
- `auth_config`: Updated auth settings
- `compliance_config`: Updated compliance
- `status`: New status

---

## Multi-Tenancy Patterns

### Tenant Identification

#### 1. Subdomain-Based
```
https://acme.auth.com  → tenant_slug = "acme"
https://globex.auth.com → tenant_slug = "globex"
```

**Implementation**:
```rust
pub fn extract_tenant_from_host(host: &str) -> Option<String> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 {
        Some(parts[0].to_string())  // First subdomain
    } else {
        None
    }
}
```

#### 2. Custom Domain
```
https://auth.acme.com → custom_domain lookup
```

**Implementation**:
```rust
pub async fn find_tenant_by_domain(domain: &str) -> Result<Option<Tenant>> {
    sqlx::query_as!(
        Tenant,
        "SELECT * FROM tenants WHERE custom_domain = ?",
        domain
    )
    .fetch_optional(&pool)
    .await
}
```

#### 3. Header-Based
```
X-Tenant-ID: 550e8400-e29b-41d4-a716-446655440000
```

**Implementation**:
```rust
pub fn extract_tenant_from_header(headers: &HeaderMap) -> Option<Uuid> {
    headers.get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
}
```

---

### Data Isolation

**All Queries Include Tenant Filter**:
```sql
SELECT * FROM users 
WHERE email = ? AND tenant_id = ?  -- Always filter by tenant

SELECT * FROM sessions 
WHERE user_id = ? AND tenant_id = ?  -- Prevent cross-tenant access
```

**Enforcement**: Database-level foreign keys + application-level checks

---

## Database Schema

```sql
CREATE TABLE tenants (
    id CHAR(36) PRIMARY KEY,
    organization_id CHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    custom_domain VARCHAR(255) UNIQUE,
    branding_config JSON DEFAULT '{}',
    auth_config JSON DEFAULT '{}',
    compliance_config JSON DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'Active',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_organization_id (organization_id),
    INDEX idx_slug (slug),
    INDEX idx_custom_domain (custom_domain),
    INDEX idx_status (status),
    
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
```

---

## Security Considerations

### Tenant Isolation

**Critical**: Prevent cross-tenant data leakage

**Enforcement**:
1. **Database Level**: All queries filter by `tenant_id`
2. **Application Level**: Middleware validates tenant access
3. **API Level**: JWT claims include `tenant_id`

**Example Vulnerability**:
```rust
// BAD: No tenant filter
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
    .fetch_one(&pool).await?;

// GOOD: Tenant filter enforced
let user = sqlx::query_as!(User, 
    "SELECT * FROM users WHERE id = ? AND tenant_id = ?", 
    user_id, tenant_id
).fetch_one(&pool).await?;
```

### Slug Validation

**Purpose**: Prevent injection attacks and ensure URL safety

**Validation**:
- Alphanumeric + hyphens only
- No leading/trailing hyphens
- Unique across platform

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

- [models/organization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/organization.rs) - Parent organization
- [models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - Tenant users

---

## Related Files

- [models/organization.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/organization.rs) - Organization entity
- [repositories/tenant_repository.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/repositories/tenant_repository.rs) - Tenant persistence
- [middleware/tenant.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware) - Tenant identification middleware

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 88  
**Security Level**: CRITICAL (multi-tenancy foundation)
