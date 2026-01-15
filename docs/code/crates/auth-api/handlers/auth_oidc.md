# handlers/auth_oidc.rs

## File Metadata

**File Path**: `crates/auth-api/src/handlers/auth_oidc.rs`  
**Crate**: `auth-api`  
**Module**: `handlers::auth_oidc`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - OIDC authentication flow

## Purpose

HTTP handlers for OpenID Connect authentication flow, enabling social login and federated authentication with identity providers.

### Problem It Solves

- OIDC login initiation
- Authorization callback handling
- CSRF protection
- Token exchange
- User provisioning from OIDC claims

---

## Detailed Code Breakdown

### Function: `login()`

**Signature**: `pub async fn login() -> impl IntoResponse`

**Purpose**: Initiate OIDC login flow

**OpenAPI Documentation**:
```rust
#[utoipa::path(
    get,
    path = "/auth/oidc/login",
    responses(
        (status = 302, description = "Redirect to identity provider")
    ),
    tag = "Authentication"
)]
```

**Process**:
1. Generate authorization URL
2. Store CSRF token and nonce in session
3. Redirect user to IdP

**Example Response**:
```http
HTTP/1.1 302 Found
Location: https://accounts.google.com/o/oauth2/v2/auth?
  client_id=...&
  redirect_uri=...&
  response_type=code&
  scope=openid+email+profile&
  state=csrf_token&
  nonce=random_nonce
```

---

### Struct: `CallbackParams`

**Purpose**: Query parameters from OIDC callback

**Fields**:
- `code`: Authorization code
- `state`: CSRF token

---

### Function: `callback()`

**Signature**: `pub async fn callback(Query(params): Query<CallbackParams>) -> impl IntoResponse`

**Purpose**: Handle OIDC callback and complete authentication

**Process**:
1. Verify CSRF token
2. Exchange code for tokens
3. Validate ID token
4. Extract user claims
5. Create or update user
6. Create session

---

## Production Implementation

### Complete OIDC Flow

```rust
use axum::{extract::{Query, State}, response::Redirect, Extension};
use auth_protocols::OidcService;

pub async fn oidc_login(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Result<Redirect> {
    // Generate authorization URL
    let (auth_url, csrf_token, nonce) = state.oidc_service.get_authorization_url();
    
    // Store in session
    session.insert("oidc_csrf", csrf_token.secret())?;
    session.insert("oidc_nonce", nonce.secret())?;
    
    Ok(Redirect::to(&auth_url))
}

pub async fn oidc_callback(
    Query(params): Query<CallbackParams>,
    Extension(session): Extension<Session>,
    State(state): State<AppState>,
) -> Result<Redirect> {
    // 1. Verify CSRF
    let stored_csrf = session.get::<String>("oidc_csrf")?
        .ok_or(AuthError::ValidationError { 
            message: "No CSRF token".to_string() 
        })?;
    
    if params.state.as_ref() != Some(&stored_csrf) {
        return Err(AuthError::ValidationError {
            message: "CSRF token mismatch".to_string(),
        });
    }
    
    // 2. Exchange code for tokens
    let token_response = state.oidc_service.client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await?;
    
    // 3. Extract and verify ID token
    let id_token = token_response.id_token()
        .ok_or(AuthError::ValidationError { 
            message: "No ID token".to_string() 
        })?;
    
    let stored_nonce = session.get::<String>("oidc_nonce")?
        .ok_or(AuthError::ValidationError { 
            message: "No nonce".to_string() 
        })?;
    
    let nonce = Nonce::new(stored_nonce);
    let claims = id_token.claims(&state.oidc_service.client.id_token_verifier(), &nonce)?;
    
    // 4. Extract user info
    let email = claims.email()
        .ok_or(AuthError::ValidationError { 
            message: "No email in claims".to_string() 
        })?
        .as_str();
    
    let name = claims.name()
        .and_then(|n| n.get(None))
        .map(|n| n.as_str());
    
    let picture = claims.picture()
        .and_then(|p| p.get(None))
        .map(|p| p.as_str().to_string());
    
    // 5. Find or create user
    let user = match state.user_repo.find_by_email(email).await? {
        Some(mut user) => {
            // Update user info from OIDC
            if let Some(name) = name {
                user.full_name = Some(name.to_string());
            }
            user.email_verified = true;
            user.avatar_url = picture;
            state.user_repo.update(user.id, user).await?
        }
        None => {
            // Create new user
            state.user_repo.create(CreateUserRequest {
                email: email.to_string(),
                full_name: name.map(String::from),
                email_verified: true,
                avatar_url: picture,
                ..Default::default()
            }).await?
        }
    };
    
    // 6. Create session
    let session_token = state.session_service.create_session(user.id, user.tenant_id).await?;
    
    // 7. Set cookie
    let cookie = format!(
        "session_token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        session_token,
        86400 * 30 // 30 days
    );
    
    Ok(Redirect::to("/dashboard")
        .with_header("Set-Cookie", cookie))
}
```

---

## Security Considerations

### 1. CSRF Protection

**Requirement**: Validate state parameter

```rust
if params.state != stored_csrf {
    return Err(AuthError::ValidationError {
        message: "CSRF attack detected".to_string(),
    });
}
```

### 2. Nonce Validation

**Requirement**: Prevent replay attacks

```rust
let claims = id_token.claims(&verifier, &nonce)?;
```

### 3. ID Token Validation

**Checks**:
- Signature verification
- Issuer validation
- Audience validation
- Expiration check

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `serde` | Deserialization |

### Internal Dependencies

- [auth-protocols/oidc.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/oidc.md) - OIDC service

---

## Related Files

- [handlers/auth.md](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/handlers/auth.md) - Standard auth
- [handlers/auth_saml.md](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/handlers/auth_saml.rs) - SAML auth

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 33  
**Security Level**: CRITICAL
