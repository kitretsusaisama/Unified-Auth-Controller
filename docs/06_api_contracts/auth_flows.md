---
title: Authentication Flow Narratives
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: API Contracts
---

# Authentication Flow Narratives

> [!NOTE]
> **Purpose**: Explain authentication flows in narrative form (NotebookLM-friendly).

---

## 1. Password Login Flow

### 1.1 Overview

The password login flow is the most basic authentication method. A user provides their email and password, which are verified against stored credentials.

### 1.2 Step-by-Step Flow

1. **User submits credentials**: User enters email and password on login form
2. **API receives request**: POST request sent to `/auth/login`
3. **User lookup**: System searches for user by email and tenant_id
4. **Status check**: Verify user is active (not suspended or locked)
5. **Password verification**: Compare provided password with stored Argon2id hash using constant-time verification
6. **Failed attempt handling**: If password incorrect:
   - Increment failed login attempts counter
   - If attempts >= 5, lock account for 30 minutes
   - Return error
7. **Success handling**: If password correct:
   - Reset failed attempts counter
   - Update last_login timestamp
   - Record login IP address
8. **Token issuance**:
   - Generate JWT access token (15-minute expiry)
   - Generate opaque refresh token (30-day expiry)
   - Store refresh token hash in database
9. **Response**: Return user object and both tokens

### 1.3 Security Considerations

- Passwords are never logged or returned in responses
- Constant-time verification prevents timing attacks
- Account lockout prevents brute force attacks
- Rate limiting (5 requests/minute) prevents credential stuffing

---

## 2. MFA Flow

### 2.1 Overview

Multi-factor authentication adds a second verification step after password login. The user must provide a time-based one-time password (TOTP) from their authenticator app.

### 2.2 Step-by-Step Flow

1. **Initial login**: User completes password login successfully
2. **MFA check**: System detects MFA is enabled for user
3. **Partial response**: Return `requires_mfa: true` without tokens
4. **MFA challenge**: User prompted to enter 6-digit TOTP code
5. **Code submission**: User submits code to `/auth/mfa/verify`
6. **Code verification**:
   - Generate expected code from stored secret
   - Compare with user-provided code (with ±1 time window tolerance)
7. **Success handling**: If code matches:
   - Issue access and refresh tokens
   - Mark MFA as verified for this session
8. **Response**: Return tokens

### 2.3 Backup Codes

If user loses access to authenticator app:
1. User selects "Use backup code" option
2. User enters one of 10 pre-generated backup codes
3. System verifies code (single-use)
4. Code is marked as used
5. Tokens issued

---

## 3. WebAuthn/Passkey Flow

### 3.1 Overview

WebAuthn provides passwordless authentication using biometrics or security keys. The flow uses public key cryptography and browser APIs.

### 3.2 Registration Flow

1. **User initiates**: User clicks "Add passkey" in settings
2. **Challenge generation**: Server generates cryptographically random challenge
3. **Browser API call**: Client calls `navigator.credentials.create()`
4. **User authentication**: User authenticates with biometric or PIN
5. **Credential creation**: Device generates key pair (private key stays on device)
6. **Public key transmission**: Public key sent to server
7. **Storage**: Server stores public key associated with user
8. **Confirmation**: User sees success message

### 3.3 Authentication Flow

1. **User initiates**: User clicks "Sign in with passkey"
2. **Challenge generation**: Server generates new random challenge
3. **Browser API call**: Client calls `navigator.credentials.get()`
4. **User authentication**: User authenticates with biometric or PIN
5. **Assertion signing**: Device signs challenge with private key
6. **Assertion transmission**: Signed assertion sent to server
7. **Verification**: Server verifies signature using stored public key
8. **Token issuance**: If valid, issue access and refresh tokens
9. **Response**: Return tokens

### 3.4 Security Advantages

- Phishing-resistant (origin validation built-in)
- No password to steal
- Private key never leaves device
- Replay protection (challenge-response)

---

## 4. Token Refresh Flow

### 4.1 Overview

When an access token expires (after 15 minutes), the client uses the refresh token to obtain a new access token without requiring the user to log in again.

### 4.2 Step-by-Step Flow

1. **Access token expires**: Client receives 401 Unauthorized
2. **Refresh request**: Client sends refresh token to `/auth/refresh`
3. **Token lookup**: Server hashes provided token and looks up in database
4. **Validation**:
   - Verify token exists
   - Verify not expired (30-day TTL)
   - Verify not revoked
5. **User lookup**: Fetch user_id and tenant_id from token record
6. **Token rotation**:
   - Delete old refresh token (single-use)
   - Generate new refresh token
   - Store new token hash in database
7. **Access token issuance**: Generate new JWT access token
8. **Response**: Return both new access token and new refresh token

### 4.3 Security Considerations

- Refresh tokens are single-use (rotation prevents replay)
- Refresh tokens are hashed in database
- If refresh token is stolen and used, legitimate user's next refresh will fail, alerting them

---

## 5. Logout Flow

### 5.1 Overview

Logout terminates the user's session and revokes their tokens.

### 5.2 Step-by-Step Flow

1. **User initiates**: User clicks "Logout"
2. **Logout request**: Client sends request to `/auth/logout` with access token
3. **Token extraction**: Server extracts JWT ID (jti) from access token
4. **Token revocation**:
   - Add jti to revoked tokens table
   - Delete refresh token from database
5. **Session termination**: Delete session record
6. **Response**: Return success
7. **Client cleanup**: Client deletes stored tokens

### 5.3 Logout All Devices

For "Logout from all devices":
1. User initiates from settings
2. Server revokes all refresh tokens for user
3. Server adds all active access token JTIs to revocation list
4. All sessions terminated
5. User must log in again on all devices

---

## 6. OIDC Flow

### 6.1 Overview

OpenID Connect (OIDC) enables single sign-on with external identity providers like Google or Azure AD.

### 6.2 Authorization Code Flow

1. **User initiates**: User clicks "Sign in with Google"
2. **Authorization request**: Client redirects to `/auth/oidc/authorize?provider=google`
3. **Provider redirect**: Server redirects to Google's authorization endpoint
4. **User authentication**: User logs in to Google (if not already)
5. **Consent**: User approves sharing profile data
6. **Authorization code**: Google redirects back with authorization code
7. **Callback**: Server receives code at `/auth/oidc/callback`
8. **Token exchange**: Server exchanges code for tokens with Google
9. **User info**: Server fetches user profile from Google
10. **User lookup/creation**:
    - If user exists (by email), link account
    - If new user, create account
11. **Token issuance**: Issue UPFlame access and refresh tokens
12. **Response**: Redirect to application with tokens

---

## 7. SAML Flow

### 7.1 Overview

SAML 2.0 enables enterprise SSO with identity providers like Okta or Azure AD.

### 7.2 SP-Initiated Flow

1. **User initiates**: User clicks "Sign in with SSO"
2. **SAML request**: Server generates SAML AuthnRequest
3. **IdP redirect**: User redirected to enterprise IdP
4. **User authentication**: User logs in to IdP (if not already)
5. **SAML response**: IdP generates signed SAML assertion
6. **ACS callback**: IdP posts assertion to `/auth/saml/acs`
7. **Assertion validation**:
   - Verify signature
   - Verify not expired
   - Verify audience matches
8. **User lookup/creation**: Extract user info from assertion
9. **Token issuance**: Issue UPFlame tokens
10. **Response**: Redirect to application

### 7.3 IdP-Initiated Flow

1. **User initiates**: User clicks app in IdP portal
2. **SAML response**: IdP sends unsolicited assertion to ACS
3. **Validation**: Same as SP-initiated
4. **Token issuance**: Issue tokens
5. **Response**: Redirect to application

---

**Document Status**: Active  
**Owner**: Engineering Team
