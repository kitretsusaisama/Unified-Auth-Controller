---
title: Enterprise Customization Guide
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Product Team
category: Multitenancy
---

# Enterprise Customization Guide

> [!NOTE]
> **Purpose**: Document what enterprises can customize in UPFlame UAC.

---

## 1. Authentication Flows

### 1.1 Custom Login Pages

**Customizable Elements**:
- Logo and branding
- Color scheme
- Custom CSS
- Welcome message
- Terms of service link

**Configuration**:
```toml
[tenant.branding]
logo_url = "https://cdn.example.com/logo.png"
primary_color = "#0066CC"
custom_css_url = "https://cdn.example.com/custom.css"
```

---

### 1.2 SSO Integrations

**Supported Providers**:
- Google Workspace
- Microsoft Azure AD
- Okta
- Custom SAML 2.0 IdP
- Custom OIDC Provider

**Per-Tenant Configuration**: Each tenant can configure their own SSO provider.

---

## 2. MFA Rules

### 2.1 MFA Enforcement

**Options**:
- Required for all users
- Required for admins only
- Optional (user choice)
- Required for specific roles

**Configuration**:
```toml
[tenant.mfa]
enforcement = "required_for_admins"
allowed_methods = ["totp", "webauthn"]
```

---

## 3. Token Claims

### 3.1 Custom Attributes

**Customizable Claims**:
- Custom user attributes
- Department
- Cost center
- Employee ID
- Custom roles

**Example JWT**:
```json
{
  "sub": "user-id",
  "tenant_id": "tenant-id",
  "custom_attrs": {
    "department": "Engineering",
    "employee_id": "EMP-12345"
  }
}
```

---

## 4. Branding

### 4.1 Visual Customization

**Elements**:
- Logo (login page, emails)
- Colors (primary, secondary, accent)
- Fonts
- Favicon

### 4.2 Email Templates

**Customizable Emails**:
- Welcome email
- Password reset
- MFA setup
- Account locked
- Security alerts

**Template Variables**:
- `{{user.name}}`
- `{{user.email}}`
- `{{tenant.name}}`
- `{{action_url}}`

---

## 5. Webhook Integrations

### 5.1 Event Webhooks

**Supported Events**:
- User registered
- User logged in
- User logged out
- Password changed
- MFA enabled/disabled
- Role assigned/revoked

**Configuration**:
```toml
[tenant.webhooks]
url = "https://api.example.com/webhooks/auth"
secret = "webhook-secret"
events = ["user.registered", "user.login"]
```

---

## 6. Custom Policies

### 6.1 ABAC Rules

**Customizable Policies**:
- Time-based access (business hours only)
- Location-based access (IP whitelist)
- Device-based access (trusted devices)
- Risk-based access (adaptive MFA)

**Example Policy**:
```json
{
  "name": "Business Hours Only",
  "rules": [
    {
      "effect": "allow",
      "conditions": [
        {"attribute": "time_of_day", "operator": "between", "values": ["09:00", "17:00"]},
        {"attribute": "day_of_week", "operator": "in", "values": ["Mon", "Tue", "Wed", "Thu", "Fri"]}
      ]
    }
  ]
}
```

---

**Document Status**: Active  
**Owner**: Product Team
