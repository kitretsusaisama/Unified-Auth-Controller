# handlers/auth_saml.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/auth_saml.rs`  
**Crate**: `auth-api`  
**Module**: `handlers::auth_saml`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - SAML authentication flow

## Purpose

HTTP handlers for SAML 2.0 authentication flow, enabling enterprise single sign-on with corporate identity providers.

### Problem It Solves

- SAML metadata generation
- Assertion Consumer Service (ACS)
- Enterprise SSO integration
- SAML response processing

---

## Detailed Code Breakdown

### Function: `metadata()`

**Signature**: `pub async fn metadata() -> impl IntoResponse`

**Purpose**: Generate and serve SAML SP metadata

**OpenAPI Documentation**:
```rust
#[utoipa::path(
    get,
    path = "/auth/saml/metadata",
    responses(
        (status = 200, description = "SAML metadata XML", content_type = "application/xml")
    ),
    tag = "Authentication"
)]
```

**Response**:
```xml
<?xml version="1.0"?>
<EntityDescriptor entityID="https://sso.example.com/saml/metadata">
  <SPSSODescriptor>
    <AssertionConsumerService 
      Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
      Location="https://sso.example.com/auth/saml/acs"
      index="0"/>
  </SPSSODescriptor>
</EntityDescriptor>
```

---

### Function: `acs()`

**Signature**: `pub async fn acs() -> impl IntoResponse`

**Purpose**: Assertion Consumer Service endpoint

**Process**:
1. Receive SAML response (POST)
2. Decode base64
3. Validate XML signature
4. Extract user attributes
5. Create or update user
6. Create session

---

## Production Implementation

### Complete SAML Flow

```rust
use axum::{response::Redirect, Form};
use auth_protocols::SamlService;

#[derive(Deserialize)]
pub struct SamlResponseParams {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

pub async fn saml_metadata(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let metadata_xml = state.saml_service.generate_metadata()?;
    
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        metadata_xml
    ))
}

pub async fn saml_acs(
    Form(params): Form<SamlResponseParams>,
    State(state): State<AppState>,
) -> Result<Redirect> {
    // 1. Decode SAML response
    let saml_response = base64::decode(&params.saml_response)?;
    let saml_xml = String::from_utf8(saml_response)?;
    
    // 2. Validate and parse assertion
    let assertion = state.saml_service.validate_response(&saml_xml)?;
    
    // 3. Extract user attributes
    let email = assertion.get_attribute("email")
        .or_else(|| assertion.get_attribute("mail"))
        .ok_or(AuthError::ValidationError {
            message: "No email in SAML assertion".to_string(),
        })?;
    
    let first_name = assertion.get_attribute("firstName")
        .or_else(|| assertion.get_attribute("givenName"));
    
    let last_name = assertion.get_attribute("lastName")
        .or_else(|| assertion.get_attribute("sn"));
    
    let full_name = match (first_name, last_name) {
        (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
        (Some(first), None) => Some(first),
        (None, Some(last)) => Some(last),
        (None, None) => assertion.get_attribute("displayName"),
    };
    
    // 4. Find or create user
    let user = match state.user_repo.find_by_email(&email).await? {
        Some(mut user) => {
            // Update user info from SAML
            if let Some(name) = full_name {
                user.full_name = Some(name);
            }
            user.email_verified = true;
            state.user_repo.update(user.id, user).await?
        }
        None => {
            // Create new user
            state.user_repo.create(CreateUserRequest {
                email,
                full_name,
                email_verified: true,
                ..Default::default()
            }).await?
        }
    };
    
    // 5. Create session
    let session_token = state.session_service.create_session(user.id, user.tenant_id).await?;
    
    // 6. Redirect to relay state or dashboard
    let redirect_url = params.relay_state.unwrap_or_else(|| "/dashboard".to_string());
    
    Ok(Redirect::to(&redirect_url)
        .with_header("Set-Cookie", format!("session_token={}", session_token)))
}
```

---

## SAML Attribute Mapping

### Common Attributes

| SAML Attribute | User Field |
|----------------|------------|
| `email`, `mail` | `email` |
| `firstName`, `givenName` | `first_name` |
| `lastName`, `sn` | `last_name` |
| `displayName` | `full_name` |
| `department` | `department` |
| `title` | `job_title` |

---

## Security Considerations

### 1. XML Signature Validation

**Requirement**: Validate SAML response signature

```rust
saml_service.validate_signature(&saml_xml)?;
```

### 2. Assertion Conditions

**Checks**:
- NotBefore / NotOnOrAfter
- Audience restriction
- Recipient validation

### 3. Replay Prevention

**Store assertion IDs** to prevent reuse

```rust
if assertion_id_exists(&assertion.id).await? {
    return Err(AuthError::ValidationError {
        message: "Assertion replay detected".to_string(),
    });
}

store_assertion_id(&assertion.id, assertion.expiry).await?;
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `base64` | Decoding |
| `serde` | Deserialization |

### Internal Dependencies

- [auth-protocols/saml.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/saml.md) - SAML service

---

## Related Files

- [handlers/auth_oidc.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth_oidc.md) - OIDC auth
- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Standard auth

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 27  
**Security Level**: CRITICAL
