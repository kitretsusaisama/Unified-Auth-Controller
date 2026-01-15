# keys.rs

## File Metadata

**File Path**: `crates/auth-crypto/src/keys.rs`  
**Crate**: `auth-crypto`  
**Module**: `keys`  
**Layer**: Infrastructure (Cryptography)  
**Security-Critical**: ✅ **YES** - Key management

## Purpose

Manages RSA key pairs for JWT signing and verification, with support for key rotation and HSM integration.

### Problem It Solves

- RSA key pair management
- Key loading from PEM files
- Key rotation
- Thread-safe key access

---

## Detailed Code Breakdown

### Struct: `KeyManager`

**Purpose**: Thread-safe RSA key manager

**Fields**:
- `encoding_key`: Private key for signing (Arc<RwLock>)
- `decoding_key`: Public key for verification (Arc<RwLock>)

---

### Method: `new()`

**Signature**: `pub async fn new() -> Result<Self, KeyError>`

**Purpose**: Create KeyManager with generated keys

**Current**: Uses test keys (production would use HSM)

---

### Method: `new_for_testing()`

**Signature**: `pub async fn new_for_testing() -> Result<Self, KeyError>`

**Purpose**: Create KeyManager with fixed test keys

**Keys**: Loaded from `test_keys/` directory

---

### Method: `from_pem_files()`

**Signature**: `pub async fn from_pem_files(private_key_path: &str, public_key_path: &str) -> Result<Self, KeyError>`

**Purpose**: Load keys from PEM files

**Example**:
```rust
let key_manager = KeyManager::from_pem_files(
    "/path/to/private_key.pem",
    "/path/to/public_key.pem"
).await?;
```

---

### Method: `get_encoding_key()`

**Signature**: `pub async fn get_encoding_key(&self) -> Result<EncodingKey, KeyError>`

**Purpose**: Get private key for JWT signing

---

### Method: `get_decoding_key()`

**Signature**: `pub async fn get_decoding_key(&self) -> Result<DecodingKey, KeyError>`

**Purpose**: Get public key for JWT verification

---

### Method: `rotate_keys()`

**Signature**: `pub async fn rotate_keys(&self) -> Result<(), KeyError>`

**Purpose**: Rotate keys (placeholder for HSM integration)

**Future Implementation**:
1. Generate new keys via HSM/KMS
2. Update keys atomically
3. Notify services of rotation

---

## Key Rotation Strategy

### Why Rotate Keys?

**Security**: Limit impact of key compromise

**Best Practice**: Rotate every 90 days

### Rotation Process

```rust
// 1. Generate new key pair
let new_key_manager = KeyManager::new().await?;

// 2. Publish new public key to JWKS endpoint
publish_jwks(new_key_manager.get_decoding_key().await?)?;

// 3. Wait for propagation (e.g., 1 hour)
tokio::time::sleep(Duration::from_secs(3600)).await;

// 4. Start signing with new key
*encoding_key.write().await = new_encoding_key;

// 5. Keep old public key for validation (grace period)
// 6. After grace period, remove old key
```

---

## Security Considerations

### 1. Private Key Protection

**Requirement**: Never expose private key

**Storage**: 
- Development: File system
- Production: HSM/KMS

### 2. Key Size

**Current**: 2048-bit RSA

**Recommendation**: 2048-bit minimum, 4096-bit for high security

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `jsonwebtoken` | Key types |
| `tokio` | Async runtime |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 120  
**Security Level**: CRITICAL
