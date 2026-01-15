---
title: MFA/WebAuthn Service Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Module Specification
crate: auth-core
---

# MFA/WebAuthn Service Specification

> [!NOTE]
> **Module**: `auth-core::services::webauthn_service`  
> **Responsibility**: Multi-factor authentication (TOTP, WebAuthn/Passkeys)

---

## 1. Overview

The **MFA/WebAuthn Service** provides multi-factor authentication capabilities including TOTP (Time-based One-Time Password) and WebAuthn/FIDO2 passkeys.

---

## 2. Supported Methods

### 2.1 TOTP (Time-based One-Time Password)

**Standard**: RFC 6238

**Features**:
- QR code generation for authenticator apps
- 6-digit codes, 30-second window
- Backup codes (10 single-use codes)

**Flow**:
1. User enables MFA
2. Generate secret key
3. Display QR code
4. User scans with authenticator app (Google Authenticator, Authy)
5. User enters code to verify
6. MFA enabled

---

### 2.2 WebAuthn/Passkeys

**Standard**: FIDO2 / WebAuthn

**Features**:
- Platform authenticators (Touch ID, Face ID, Windows Hello)
- Security keys (YubiKey, etc.)
- Passkey registration and authentication
- Multiple passkeys per user

**Flow**:
1. User initiates passkey registration
2. Server generates challenge
3. Browser calls `navigator.credentials.create()`
4. User authenticates (biometric, PIN)
5. Public key sent to server
6. Server stores public key

---

## 3. Public API

### 3.1 TOTP Operations

```rust
impl WebAuthnService {
    pub async fn enable_totp(&self, user_id: Uuid) 
        -> Result<TotpSetup, AuthError>;
    
    pub async fn verify_totp(&self, user_id: Uuid, code: &str) 
        -> Result<bool, AuthError>;
    
    pub async fn generate_backup_codes(&self, user_id: Uuid) 
        -> Result<Vec<String>, AuthError>;
}
```

### 3.2 WebAuthn Operations

```rust
impl WebAuthnService {
    pub async fn start_registration(&self, user_id: Uuid) 
        -> Result<CredentialCreationOptions, AuthError>;
    
    pub async fn finish_registration(&self, user_id: Uuid, credential: PublicKeyCredential) 
        -> Result<(), AuthError>;
    
    pub async fn start_authentication(&self, user_id: Uuid) 
        -> Result<CredentialRequestOptions, AuthError>;
    
    pub async fn finish_authentication(&self, user_id: Uuid, assertion: PublicKeyCredential) 
        -> Result<bool, AuthError>;
}
```

---

## 4. Security Considerations

### 4.1 TOTP Security

- **Secret Storage**: Encrypted at rest
- **Time Sync**: 30-second window with ±1 window tolerance
- **Backup Codes**: Single-use, hashed storage

### 4.2 WebAuthn Security

- **Challenge**: Cryptographically random, single-use
- **Public Key Storage**: Only public key stored (never private)
- **Phishing Resistant**: Origin validation built-in
- **Replay Protection**: Challenge-response mechanism

---

## 5. Examples

### 5.1 Enable TOTP

```rust
let setup = webauthn_service.enable_totp(user_id).await?;
println!("Secret: {}", setup.secret);
println!("QR Code: {}", setup.qr_code_url);

// User scans QR code and enters code
let verified = webauthn_service.verify_totp(user_id, "123456").await?;
```

### 5.2 Register Passkey

```rust
// Server-side
let options = webauthn_service.start_registration(user_id).await?;

// Client-side (JavaScript)
// const credential = await navigator.credentials.create(options);

// Server-side
webauthn_service.finish_registration(user_id, credential).await?;
```

---

**Document Status**: Active  
**Owner**: Engineering Team
