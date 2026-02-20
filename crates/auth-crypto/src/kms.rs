use anyhow::Result;
use async_trait::async_trait;
use rand::rngs::OsRng;
use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

#[async_trait]
pub trait KeyProvider: Send + Sync {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool>;
    fn public_key_pem(&self) -> String;
}

pub struct SoftKeyProvider {
    key: RsaPrivateKey,
}

impl SoftKeyProvider {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let bits = 2048;
        let key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        Self { key }
    }
}

#[async_trait]
impl KeyProvider for SoftKeyProvider {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Hash the data first
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();

        // Sign the hash using PKCS1v15
        let padding = Pkcs1v15Sign::new::<Sha256>();
        let signature = self.key.sign_with_rng(&mut OsRng, padding, &digest)?;
        Ok(signature.to_vec())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        // Hash the data
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();

        let public_key = RsaPublicKey::from(&self.key);
        let padding = Pkcs1v15Sign::new::<Sha256>();

        match public_key.verify(padding, &digest, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn public_key_pem(&self) -> String {
        let public_key = RsaPublicKey::from(&self.key);
        public_key
            .to_public_key_pem(LineEnding::LF)
            .unwrap_or_default()
    }
}

pub struct HsmKeyProvider {
    // Stub for HSM connection (e.g. PKCS#11)
    _slot_id: u64,
}

impl HsmKeyProvider {
    pub fn new(slot_id: u64) -> Self {
        Self { _slot_id: slot_id }
    }
}

#[async_trait]
impl KeyProvider for HsmKeyProvider {
    async fn sign(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // In real impl, send command to HSM
        Ok(vec![0u8; 64]) // Dummy signature
    }

    async fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool> {
        Ok(true) // Dummy verification
    }

    fn public_key_pem(&self) -> String {
        "-----BEGIN PUBLIC KEY-----\nMOCK_HSM_KEY\n-----END PUBLIC KEY-----".to_string()
    }
}

impl Default for SoftKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}
