use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passkey;

impl Passkey {
    pub fn cred_id(&self) -> Uuid {
        Uuid::nil()
    }
}

// Dummy types for return signature
#[derive(Debug, Clone)]
pub struct CreationChallengeResponse;
#[derive(Debug, Clone)]
pub struct RegisterPublicKeyCredentialCreationOptions;
#[derive(Debug, Clone)]
pub struct PasskeyRegistration;
#[derive(Debug, Clone)]
pub struct RegisterPublicKeyCredential;

#[async_trait]
pub trait WebauthnStore: Send + Sync {
    async fn save_passkey(&self, user_id: Uuid, passkey: &Passkey) -> anyhow::Result<()>;
}

pub struct WebauthnService {
    store: Arc<dyn WebauthnStore>,
}

impl WebauthnService {
    pub fn new(store: Arc<dyn WebauthnStore>, _rp_origin: &str, _rp_id: &str) -> Self {
        Self { store }
    }

    pub async fn start_registration(&self, _user_id: Uuid, _username: &str) -> anyhow::Result<(CreationChallengeResponse, RegisterPublicKeyCredentialCreationOptions)> {
        Ok((CreationChallengeResponse, RegisterPublicKeyCredentialCreationOptions))
    }

    pub async fn finish_registration(&self, user_id: Uuid, _challenge: &PasskeyRegistration, _response: &RegisterPublicKeyCredential) -> anyhow::Result<()> {
        let passkey = Passkey;
        self.store.save_passkey(user_id, &passkey).await?;
        Ok(())
    }
}
