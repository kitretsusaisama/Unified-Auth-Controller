//! Key management for JWT signing and verification

use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use rsa::{RsaPrivateKey, RsaPublicKey, traits::PublicKeyParts, pkcs1::EncodeRsaPublicKey, pkcs1::EncodeRsaPrivateKey, pkcs1::DecodeRsaPublicKey};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::thread_rng;

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("Key generation failed: {0}")]
    GenerationError(String),
    #[error("Key loading failed: {0}")]
    LoadingError(String),
    #[error("Invalid key format: {0}")]
    InvalidFormat(String),
}

#[derive(Clone)]
pub struct KeyManager {
    #[allow(dead_code)]
    encoding_key: Arc<RwLock<EncodingKey>>,
    #[allow(dead_code)]
    decoding_key: Arc<RwLock<DecodingKey>>,
    // Store PEMs for reconstruction if needed, or simply for JWK generation
    private_key_pem: String,
    public_key_pem: String,
}

impl KeyManager {
    /// Create a new KeyManager with generated RSA keys
    pub async fn new() -> Result<Self, KeyError> {
        // For now, use the test keys. In production, this would generate new keys or load from HSM
        Self::new_for_testing().await
    }

    /// Create a KeyManager for testing with dynamically generated keys
    pub async fn new_for_testing() -> Result<Self, KeyError> {
        let mut rng = thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| KeyError::GenerationError(e.to_string()))?;
        let public_key = RsaPublicKey::from(&private_key);

        let private_pem = private_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| KeyError::GenerationError(e.to_string()))?;
        let public_pem = public_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| KeyError::GenerationError(e.to_string()))?;

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;
        
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;

        Ok(Self {
            encoding_key: Arc::new(RwLock::new(encoding_key)),
            decoding_key: Arc::new(RwLock::new(decoding_key)),
            private_key_pem: private_pem.to_string(),
            public_key_pem: public_pem.to_string(),
        })
    }

    /// Load KeyManager from PEM files
    pub async fn from_pem_files(private_key_path: &str, public_key_path: &str) -> Result<Self, KeyError> {
        let private_key_pem = tokio::fs::read_to_string(private_key_path).await
            .map_err(|e| KeyError::LoadingError(format!("Failed to read private key: {}", e)))?;
        
        let public_key_pem = tokio::fs::read_to_string(public_key_path).await
            .map_err(|e| KeyError::LoadingError(format!("Failed to read public key: {}", e)))?;

        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;
        
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;

        Ok(Self {
            encoding_key: Arc::new(RwLock::new(encoding_key)),
            decoding_key: Arc::new(RwLock::new(decoding_key)),
            private_key_pem,
            public_key_pem,
        })
    }

    /// Get the encoding key for JWT signing
    pub async fn get_encoding_key(&self) -> Result<EncodingKey, KeyError> {
        // We recreate from the stored PEM because EncodingKey is not easily clonable directly
        // without internal access, and existing implementation suggested this pattern.
        // Or we could return a clone of the key if we change the struct or if EncodingKey supported it.
        // But since we store the PEM now, we can just use it.
        EncodingKey::from_rsa_pem(self.private_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))
    }

    /// Get the decoding key for JWT verification
    pub async fn get_decoding_key(&self) -> Result<DecodingKey, KeyError> {
        DecodingKey::from_rsa_pem(self.public_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))
    }

    /// Rotate keys (for future HSM integration)
    pub async fn rotate_keys(&self) -> Result<(), KeyError> {
        // For now, this is a placeholder. In production, this would:
        // 1. Generate new keys via HSM/KMS
        // 2. Update the keys atomically
        // 3. Notify all services of the key rotation
        tracing::warn!("Key rotation not yet implemented - using fixed test keys");
        Ok(())
    }

    /// Get the JWK Set (public keys)
    pub fn get_jwk_set(&self) -> serde_json::Value {
        // Parse public key PEM
        let pub_key = RsaPublicKey::from_pkcs1_pem(&self.public_key_pem).expect("Invalid Public Key PEM");

        let n = URL_SAFE_NO_PAD.encode(pub_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(pub_key.e().to_bytes_be());

        serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "kid": "auth-core-key-1",
                "alg": "RS256",
                "n": n,
                "e": e
            }]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_generation() {
        let key_manager = KeyManager::new().await.unwrap();
        
        // Should be able to get keys without error
        let _encoding_key = key_manager.get_encoding_key().await.unwrap();
        let _decoding_key = key_manager.get_decoding_key().await.unwrap();
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let key_manager = KeyManager::new().await.unwrap();
        
        // Rotate keys (currently a no-op)
        key_manager.rotate_keys().await.unwrap();
    }
}
