# lib.rs

## File Metadata

**File Path**: `crates/auth-protocols/src/lib.rs`  
**Crate**: `auth-protocols`  
**Module**: Root module  
**Layer**: Adapter (Module Aggregator)  
**Security-Critical**: ❌ **NO** - Module organization

## Purpose

Module aggregator that organizes and re-exports all authentication protocol implementations, providing a clean public API for the auth-protocols crate.

---

## Module Structure

### Submodules

```rust
pub mod oidc;
pub mod saml;
pub mod oauth;
```

### Re-exports

```rust
pub use oidc::OidcService;
pub use saml::SamlService;
pub use oauth::OAuthService;
```

---

## Protocol Overview

### 1. OpenID Connect (OIDC)

**Module**: `oidc`  
**Purpose**: Federated authentication with identity layer  
**Use Cases**: Social login, enterprise SSO  
**Providers**: Google, Microsoft, Okta, Auth0

### 2. SAML 2.0

**Module**: `saml`  
**Purpose**: Enterprise single sign-on  
**Use Cases**: Corporate identity federation  
**Providers**: Active Directory, Okta, OneLogin

### 3. OAuth 2.0

**Module**: `oauth`  
**Purpose**: Delegated authorization  
**Use Cases**: API access, third-party integrations  
**Providers**: GitHub, GitLab, Bitbucket

### 4. SCIM 2.0

**Module**: `scim` (placeholder)  
**Purpose**: User provisioning  
**Use Cases**: Automated onboarding/offboarding  
**Providers**: Okta, Azure AD, OneLogin

---

## Usage

### Import Pattern

```rust
// Instead of:
use auth_protocols::oidc::OidcService;
use auth_protocols::saml::SamlService;
use auth_protocols::oauth::OAuthService;

// Use:
use auth_protocols::{OidcService, SamlService, OAuthService};
```

---

## Protocol Comparison

| Feature | OIDC | SAML | OAuth 2.0 |
|---------|------|------|-----------|
| **Primary Use** | Authentication | Authentication | Authorization |
| **Format** | JSON/JWT | XML | JSON |
| **Complexity** | Medium | High | Low |
| **Mobile Support** | Excellent | Poor | Excellent |
| **Enterprise Adoption** | Growing | Dominant | Universal |
| **Token Type** | ID Token + Access Token | SAML Assertion | Access Token |

---

## Related Files

- [oidc.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/oidc.md) - OpenID Connect
- [saml.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/saml.md) - SAML 2.0
- [oauth.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/oauth.md) - OAuth 2.0
- [scim.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-protocols/scim.md) - SCIM 2.0

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 7  
**Security Level**: LOW
