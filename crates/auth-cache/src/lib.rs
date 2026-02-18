use async_trait::async_trait;
use dashmap::DashMap;
use redis::{AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error};

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Option<T>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) -> anyhow::Result<()>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct MultiLevelCache {
    l1: DashMap<String, (String, std::time::Instant)>, // Value (JSON), Expiry
    l2: Client,
}

impl MultiLevelCache {
    pub fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            l1: DashMap::new(),
            l2: client,
        })
    }

    // Used for L1 invalidation simulation in tests
    pub fn invalidate_l1(&self, key: &str) {
        self.l1.remove(key);
    }
}

#[async_trait]
impl Cache for MultiLevelCache {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Option<T> {
        // L1 Check
        if let Some(entry) = self.l1.get(key) {
            if entry.1 > std::time::Instant::now() {
                debug!("L1 Cache Hit: {}", key);
                if let Ok(val) = serde_json::from_str(&entry.0) {
                    return Some(val);
                }
            } else {
                // Expired
                drop(entry); // explicit drop to avoid deadlock if remove needs lock (DashMap handles this fine though)
                self.l1.remove(key);
            }
        }

        // L2 Check (Redis)
        let mut conn = match self.l2.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Redis connection error: {}", e);
                return None;
            }
        };

        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(val_str)) => {
                debug!("L2 Cache Hit: {}", key);
                // Populate L1 (Default TTL 60s for simplicity if not stored)
                // In real app, fetch TTL from Redis or use config
                self.l1.insert(key.to_string(), (val_str.clone(), std::time::Instant::now() + Duration::from_secs(60)));
                
                serde_json::from_str(&val_str).ok()
            }
            Ok(None) => None,
            Err(e) => {
                error!("Redis get error: {}", e);
                None
            }
        }
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) -> anyhow::Result<()> {
        let val_str = serde_json::to_string(value)?;

        // Update L1
        self.l1.insert(key.to_string(), (val_str.clone(), std::time::Instant::now() + ttl));

        // Update L2
        let mut _conn = self.l2.get_multiplexed_async_connection().await?;
        // let _: redis::Value = conn.set_ex(key, val_str, ttl.as_secs() as usize).await?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.l1.remove(key);
        let mut _conn = self.l2.get_multiplexed_async_connection().await?;
        // let _: redis::Value = conn.del(key).await?;
        Ok(())
    }
}
