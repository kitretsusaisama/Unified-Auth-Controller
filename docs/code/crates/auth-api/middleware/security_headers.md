# middleware/security_headers.rs

## File Metadata

**File Path**: `crates/auth-api/src/middleware/security_headers.rs`  
**Crate**: `auth-api`  
**Module**: `middleware::security_headers`  
**Layer**: Adapter (HTTP)  
**Security-Critical**: ✅ **YES** - HTTP security headers

## Purpose

Axum middleware that adds comprehensive security headers to all HTTP responses, protecting against common web vulnerabilities including clickjacking, XSS, MIME sniffing, and more.

### Problem It Solves

- Prevents clickjacking attacks
- Blocks MIME type sniffing
- Enables XSS protection
- Enforces HTTPS usage
- Implements Content Security Policy
- Controls browser permissions

---

## Detailed Code Breakdown

### Function: `security_headers_middleware()`

**Signature**: `pub async fn security_headers_middleware(req: Request, next: Next) -> Response`

**Purpose**: Add security headers to all responses

**Pattern**: Axum middleware

---

## Security Headers Applied

### 1. X-Frame-Options

```rust
headers.insert(
    "x-frame-options",
    HeaderValue::from_static("DENY"),
);
```

**Purpose**: Prevents clickjacking attacks  
**Value**: `DENY`  
**Effect**: Page cannot be displayed in `<iframe>`, `<frame>`, `<embed>`, or `<object>`

**Options**:
- `DENY` - Never allow framing (most secure)
- `SAMEORIGIN` - Allow framing from same origin
- `ALLOW-FROM uri` - Allow specific URI (deprecated)

**Attack Prevented**:
```html
<!-- Attacker site -->
<iframe src="https://your-app.com/transfer-money"></iframe>
<!-- User clicks thinking it's attacker's button -->
```

---

### 2. X-Content-Type-Options

```rust
headers.insert(
    "x-content-type-options",
    HeaderValue::from_static("nosniff"),
);
```

**Purpose**: Prevents MIME type sniffing  
**Value**: `nosniff`  
**Effect**: Browser must respect `Content-Type` header

**Attack Prevented**:
```
// Attacker uploads "image.jpg" containing JavaScript
// Without nosniff: Browser might execute it as JS
// With nosniff: Browser treats it strictly as image
```

---

### 3. X-XSS-Protection

```rust
headers.insert(
    "x-xss-protection",
    HeaderValue::from_static("1; mode=block"),
);
```

**Purpose**: Enable browser XSS filter  
**Value**: `1; mode=block`  
**Effect**: Browser blocks page if XSS detected

**Note**: Legacy header, CSP is preferred modern approach

---

### 4. Strict-Transport-Security (HSTS)

```rust
headers.insert(
    "strict-transport-security",
    HeaderValue::from_static("max-age=31536000; includeSubDomains"),
);
```

**Purpose**: Enforce HTTPS connections  
**Value**: `max-age=31536000; includeSubDomains`  
**Effect**: 
- Browser only connects via HTTPS for 1 year
- Applies to all subdomains

**Parameters**:
- `max-age=31536000` - 365 days in seconds
- `includeSubDomains` - Apply to all subdomains
- Optional: `preload` - Include in browser preload list

**Security**:
- Prevents SSL stripping attacks
- Protects against downgrade attacks
- First visit must be HTTPS

---

### 5. Content-Security-Policy (CSP)

```rust
headers.insert(
    "content-security-policy",
    HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
);
```

**Purpose**: Control resource loading  
**Value**: `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'`

**Directives**:

| Directive | Value | Effect |
|-----------|-------|--------|
| `default-src` | `'self'` | Only load resources from same origin |
| `script-src` | `'self' 'unsafe-inline'` | Scripts from same origin + inline |
| `style-src` | `'self' 'unsafe-inline'` | Styles from same origin + inline |

**Production Recommendation**:
```rust
// Remove 'unsafe-inline' for better security
"default-src 'self'; \
 script-src 'self'; \
 style-src 'self'; \
 img-src 'self' data: https:; \
 font-src 'self'; \
 connect-src 'self'; \
 frame-ancestors 'none'; \
 base-uri 'self'; \
 form-action 'self'"
```

---

### 6. Referrer-Policy

```rust
headers.insert(
    "referrer-policy",
    HeaderValue::from_static("strict-origin-when-cross-origin"),
);
```

**Purpose**: Control referrer information  
**Value**: `strict-origin-when-cross-origin`  
**Effect**: 
- Same origin: Send full URL
- Cross-origin HTTPS→HTTPS: Send origin only
- Cross-origin HTTPS→HTTP: Send nothing

**Options**:
- `no-referrer` - Never send referrer
- `same-origin` - Only for same origin
- `strict-origin` - Only origin, HTTPS→HTTP blocked
- `strict-origin-when-cross-origin` - Recommended

---

### 7. Permissions-Policy

```rust
headers.insert(
    "permissions-policy",
    HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
);
```

**Purpose**: Control browser features  
**Value**: `geolocation=(), microphone=(), camera=()`  
**Effect**: Disable geolocation, microphone, and camera access

**Syntax**: `feature=(allowed-origins)`
- `()` - Disabled for all
- `self` - Allowed for same origin
- `*` - Allowed for all origins

**Common Features**:
```rust
"geolocation=(), \
 microphone=(), \
 camera=(), \
 payment=(), \
 usb=(), \
 magnetometer=(), \
 gyroscope=(), \
 accelerometer=()"
```

---

## Integration

### Axum Router Setup

```rust
use axum::{Router, middleware};
use crate::middleware::security_headers::security_headers_middleware;

pub fn app() -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/auth/login", post(login))
        // Apply security headers to all routes
        .layer(middleware::from_fn(security_headers_middleware))
}
```

---

## Testing

### Manual Testing

```bash
# Test security headers
curl -I https://localhost:8080/api/health

# Expected headers:
# x-frame-options: DENY
# x-content-type-options: nosniff
# x-xss-protection: 1; mode=block
# strict-transport-security: max-age=31536000; includeSubDomains
# content-security-policy: default-src 'self'; ...
# referrer-policy: strict-origin-when-cross-origin
# permissions-policy: geolocation=(), microphone=(), camera=()
```

### Automated Testing

```rust
#[tokio::test]
async fn test_security_headers() {
    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(middleware::from_fn(security_headers_middleware));
    
    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    let headers = response.headers();
    
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("strict-transport-security").unwrap(), 
               "max-age=31536000; includeSubDomains");
}
```

---

## Security Best Practices

### 1. CSP Nonce for Inline Scripts

**Problem**: `'unsafe-inline'` weakens CSP

**Solution**: Use nonces
```rust
// Generate nonce per request
let nonce = generate_nonce();

headers.insert(
    "content-security-policy",
    HeaderValue::from_str(&format!(
        "default-src 'self'; script-src 'self' 'nonce-{}'",
        nonce
    )).unwrap(),
);

// In HTML
// <script nonce="{nonce}">...</script>
```

### 2. HSTS Preloading

**Steps**:
1. Add `preload` directive
   ```rust
   "max-age=31536000; includeSubDomains; preload"
   ```
2. Submit to https://hstspreload.org/
3. Wait for browser inclusion

### 3. Environment-Specific CSP

```rust
pub fn get_csp_header(env: &str) -> &'static str {
    match env {
        "development" => "default-src 'self' 'unsafe-inline' 'unsafe-eval'",
        "production" => "default-src 'self'; script-src 'self'; style-src 'self'",
        _ => "default-src 'none'",
    }
}
```

---

## Security Scanning

### Tools

1. **Mozilla Observatory**
   ```bash
   # Scan your site
   https://observatory.mozilla.org/
   ```

2. **SecurityHeaders.com**
   ```bash
   https://securityheaders.com/?q=your-domain.com
   ```

3. **OWASP ZAP**
   - Automated security testing
   - Header validation

---

## Common Issues

### Issue 1: HSTS on HTTP

**Problem**: HSTS header on HTTP is ignored

**Solution**: Only send on HTTPS
```rust
if req.uri().scheme_str() == Some("https") {
    headers.insert("strict-transport-security", ...);
}
```

### Issue 2: CSP Breaking Functionality

**Problem**: Strict CSP blocks legitimate resources

**Solution**: Use CSP report-only mode first
```rust
headers.insert(
    "content-security-policy-report-only",
    HeaderValue::from_static("..."),
);
```

### Issue 3: Frame-Options vs CSP

**Problem**: Both control framing

**Solution**: Use CSP `frame-ancestors` (preferred)
```rust
"frame-ancestors 'none'"  // Equivalent to X-Frame-Options: DENY
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |

---

## Related Files

- [middleware/rate_limit.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/rate_limit.rs) - Rate limiting
- [middleware/request_id.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-api/src/middleware/request_id.rs) - Request tracking
- [lib.rs](file:///c:/Users/Victo/Downloads/sso/docs/code/crates/auth-api/lib.md) - API setup

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 60  
**Security Level**: CRITICAL
