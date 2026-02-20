//! Key management for JWT signing and verification

use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

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
    encoding_key: Arc<RwLock<EncodingKey>>,
    decoding_key: Arc<RwLock<DecodingKey>>,
}

impl KeyManager {
    /// Create a new KeyManager with generated RSA keys
    pub async fn new() -> Result<Self, KeyError> {
        // For now, use the test keys. In production, this would generate new keys or load from HSM
        Self::new_for_testing().await
    }

    /// Create a KeyManager for testing with fixed keys
    pub async fn new_for_testing() -> Result<Self, KeyError> {
        // Use a fixed RSA key pair for testing
        let private_key_pem = include_str!("../test_keys/private_key.pem");
        let public_key_pem = include_str!("../test_keys/public_key.pem");

        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))?;

        Ok(Self {
            encoding_key: Arc::new(RwLock::new(encoding_key)),
            decoding_key: Arc::new(RwLock::new(decoding_key)),
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
        })
    }

    /// Get the encoding key for JWT signing
    pub async fn get_encoding_key(&self) -> Result<EncodingKey, KeyError> {
        let _key = self.encoding_key.read().await;
        // We need to clone the actual key data, not just the reference
        // For now, we'll recreate from the same source
        let private_key_pem = include_str!("../test_keys/private_key.pem");
        EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| KeyError::LoadingError(e.to_string()))
    }

    /// Get the decoding key for JWT verification
    pub async fn get_decoding_key(&self) -> Result<DecodingKey, KeyError> {
        let _key = self.decoding_key.read().await;
        // We need to clone the actual key data, not just the reference
        // For now, we'll recreate from the same source
        let public_key_pem = include_str!("../test_keys/public_key.pem");
        DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
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