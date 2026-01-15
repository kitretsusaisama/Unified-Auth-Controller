# kms.rs

## File Metadata

**File Path**: `crates/auth-crypto/src/kms.rs`  
**Crate**: `auth-crypto`  
**Module**: `kms`  
**Layer**: Infrastructure (Cryptography)  
**Security-Critical**: ✅ **YES** - Key management system

## Purpose

Provides key provider abstraction for software and hardware security module (HSM) based cryptographic operations.

### Problem It Solves

- Abstraction over key storage
- Software key provider
- HSM integration
- Digital signatures
- Signature verification

---

## Detailed Code Breakdown

### Trait: `KeyProvider`

**Purpose**: Abstract key provider interface

**Methods**:
```rust
async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool>;
fn public_key_pem(&self) -> String;
```

---

### Struct: `SoftKeyProvider`

**Purpose**: Software-based RSA key provider

**Fields**:
- `key`: RSA private key (2048-bit)

---

### Method: `SoftKeyProvider::sign()`

**Process**:
1. Hash data with SHA-256
2. Sign hash with RSA private key (PKCS1v15)
3. Return signature

**Example**:
```rust
let provider = SoftKeyProvider::new();
let data = b"message to sign";
let signature = provider.sign(data).await?;
```

---

### Method: `SoftKeyProvider::verify()`

**Process**:
1. Hash data with SHA-256
2. Verify signature using RSA public key
3. Return verification result

---

### Struct: `HsmKeyProvider`

**Purpose**: HSM-based key provider (stub)

**Fields**:
- `_slot_id`: HSM slot identifier

**Note**: Currently a placeholder for future HSM integration

---

## Production HSM Integration

### AWS KMS Example

```rust
use aws_sdk_kms::Client as KmsClient;

pub struct AwsKmsProvider {
    client: KmsClient,
    key_id: String,
}

impl AwsKmsProvider {
    pub async fn new(key_id: String) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = KmsClient::new(&config);
        Ok(Self { client, key_id })
    }
}

#[async_trait]
impl KeyProvider for AwsKmsProvider {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let response = self.client
            .sign()
            .key_id(&self.key_id)
            .message(Blob::new(data))
            .signing_algorithm(SigningAlgorithmSpec::RsassaPkcs1V15Sha256)
            .send()
            .await?;
        
        Ok(response.signature.unwrap().into_inner())
    }
    
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let response = self.client
            .verify()
            .key_id(&self.key_id)
            .message(Blob::new(data))
            .signature(Blob::new(signature))
            .signing_algorithm(SigningAlgorithmSpec::RsassaPkcs1V15Sha256)
            .send()
            .await?;
        
        Ok(response.signature_valid)
    }
    
    fn public_key_pem(&self) -> String {
        // Fetch from KMS
        unimplemented!()
    }
}
```

---

## Security Considerations

### 1. Key Storage

**Software**: Keys in memory (development only)
**HSM**: Keys never leave hardware (production)

### 2. Signature Algorithm

**Current**: RSA-SHA256 (PKCS1v15)
**Alternative**: RSA-PSS (more secure padding)

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `rsa` | RSA operations |
| `sha2` | SHA-256 hashing |
| `rand` | Random number generation |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 89  
**Security Level**: CRITICAL
