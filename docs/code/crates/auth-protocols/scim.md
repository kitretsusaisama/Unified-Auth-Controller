# scim.rs

## File Metadata

**File Path**: `crates/auth-protocols/src/scim.rs`  
**Crate**: `auth-protocols`  
**Module**: `scim`  
**Layer**: Adapter (Protocol)  
**Security-Critical**: ⚠️ **MEDIUM** - User provisioning

## Purpose

Placeholder for SCIM 2.0 (System for Cross-domain Identity Management) protocol implementation for automated user provisioning and deprovisioning.

### Problem It Solves

- Automated user provisioning
- Cross-system identity synchronization
- User lifecycle management
- Group management
- Enterprise directory integration

---

## SCIM 2.0 Overview

### What is SCIM?

SCIM is a standard for automating the exchange of user identity information between identity domains or IT systems.

### Use Cases

1. **Automated Onboarding**: Create user accounts automatically when employee joins
2. **Automated Offboarding**: Disable accounts when employee leaves
3. **Attribute Sync**: Keep user attributes synchronized across systems
4. **Group Management**: Manage group memberships centrally

---

## Future Implementation

### SCIM Resources

#### User Resource

```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "id": "2819c223-7f76-453a-919d-413861904646",
  "userName": "user@example.com",
  "name": {
    "formatted": "John Doe",
    "familyName": "Doe",
    "givenName": "John"
  },
  "emails": [
    {
      "value": "user@example.com",
      "type": "work",
      "primary": true
    }
  ],
  "active": true
}
```

#### Group Resource

```json
{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:Group"],
  "id": "e9e30dba-f08f-4109-8486-d5c6a331660a",
  "displayName": "Engineering",
  "members": [
    {
      "value": "2819c223-7f76-453a-919d-413861904646",
      "display": "John Doe"
    }
  ]
}
```

---

## SCIM Operations

### 1. Create User (POST /Users)

```http
POST /scim/v2/Users
Content-Type: application/scim+json

{
  "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
  "userName": "newuser@example.com",
  "name": {
    "givenName": "Jane",
    "familyName": "Smith"
  },
  "emails": [
    {
      "value": "newuser@example.com",
      "primary": true
    }
  ],
  "active": true
}
```

### 2. Update User (PATCH /Users/{id})

```http
PATCH /scim/v2/Users/2819c223-7f76-453a-919d-413861904646
Content-Type: application/scim+json

{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
  "Operations": [
    {
      "op": "replace",
      "path": "active",
      "value": false
    }
  ]
}
```

### 3. Delete User (DELETE /Users/{id})

```http
DELETE /scim/v2/Users/2819c223-7f76-453a-919d-413861904646
```

### 4. List Users (GET /Users)

```http
GET /scim/v2/Users?filter=userName eq "user@example.com"
```

---

## Implementation Plan

### Struct: `ScimService`

```rust
pub struct ScimService {
    user_repo: Arc<dyn UserRepository>,
    group_repo: Arc<dyn GroupRepository>,
}

impl ScimService {
    pub async fn create_user(&self, scim_user: ScimUser) -> Result<ScimUser> {
        // Convert SCIM user to internal user model
        // Create user in database
        // Return SCIM representation
    }
    
    pub async fn update_user(&self, id: Uuid, patch: ScimPatch) -> Result<ScimUser> {
        // Apply SCIM patch operations
        // Update user in database
        // Return updated SCIM representation
    }
    
    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        // Soft delete or hard delete user
    }
    
    pub async fn list_users(&self, filter: Option<String>) -> Result<ScimListResponse> {
        // Parse SCIM filter
        // Query users
        // Return paginated SCIM response
    }
}
```

---

## Integration Examples

### Example 1: Okta SCIM Integration

```rust
// Okta sends SCIM requests to provision users
// POST /scim/v2/Users
pub async fn scim_create_user(
    Json(scim_user): Json<ScimUser>,
    State(state): State<AppState>,
) -> Result<Json<ScimUser>> {
    let user = state.scim_service.create_user(scim_user).await?;
    Ok(Json(user))
}
```

### Example 2: Azure AD SCIM Integration

```rust
// Azure AD syncs users via SCIM
// PATCH /scim/v2/Users/{id}
pub async fn scim_update_user(
    Path(id): Path<Uuid>,
    Json(patch): Json<ScimPatch>,
    State(state): State<AppState>,
) -> Result<Json<ScimUser>> {
    let user = state.scim_service.update_user(id, patch).await?;
    Ok(Json(user))
}
```

---

## Security Considerations

### 1. Authentication

**Requirement**: OAuth 2.0 bearer tokens

```http
Authorization: Bearer <access_token>
```

### 2. Authorization

**Requirement**: Verify SCIM client permissions

### 3. Rate Limiting

**Requirement**: Protect against bulk operations

---

## Dependencies

### External Crates (Future)

| Crate | Purpose |
|-------|---------|
| `scim-rs` | SCIM protocol implementation |
| `serde` | JSON serialization |

---

## Related Files

- [oidc.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/oidc.md) - OpenID Connect
- [saml.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/saml.md) - SAML 2.0

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 3  
**Security Level**: MEDIUM
