use async_trait::async_trait;
use dashmap::DashMap;
use redis::{AsyncCommands, Client};
use std::time::Duration;
use tracing::debug;

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>>;
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

pub struct MultiLevelCache {
    l1: DashMap<String, (String, std::time::Instant)>, // Value (JSON), Expiry
    l2: Option<Client>,
}

impl MultiLevelCache {
    pub fn new(redis_url: Option<String>) -> anyhow::Result<Self> {
        let client = if let Some(url) = redis_url {
            Some(Client::open(url)?)
        } else {
            None
        };

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
    async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        // L1 Check
        if let Some(entry) = self.l1.get(key) {
            if entry.1 > std::time::Instant::now() {
                debug!("L1 Cache Hit: {}", key);
                return Ok(Some(entry.0.clone()));
            } else {
                // Expired
                drop(entry);
                self.l1.remove(key);
            }
        }

        // L2 Check (Redis)
        if let Some(client) = &self.l2 {
            let mut conn = client.get_multiplexed_async_connection().await?;

            match conn.get::<_, Option<String>>(key).await? {
                Some(val_str) => {
                    debug!("L2 Cache Hit: {}", key);
                    // Populate L1 (Default TTL 60s)
                    self.l1.insert(
                        key.to_string(),
                        (
                            val_str.clone(),
                            std::time::Instant::now() + Duration::from_secs(60),
                        ),
                    );

                    Ok(Some(val_str))
                }
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> anyhow::Result<()> {
        // Update L1
        self.l1.insert(
            key.to_string(),
            (value.to_string(), std::time::Instant::now() + ttl),
        );

        // Update L2
        if let Some(client) = &self.l2 {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let _: redis::Value = conn.set_ex(key, value, ttl.as_secs()).await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        self.l1.remove(key);
        if let Some(client) = &self.l2 {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let _: redis::Value = conn.del(key).await?;
        }
        Ok(())
    }
}
