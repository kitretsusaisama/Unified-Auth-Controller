use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use sqlx::MySqlPool;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone)]
pub struct ShardConfig {
    pub shard_id: u32,
    pub database_url: String,
    // Add weight for consistent hashing distribution if needed
    pub weight: u32,
}

pub struct ShardManager {
    // Map of Shard ID to connection pool
    pools: RwLock<HashMap<u32, MySqlPool>>,
    // Consistent hashing ring (simplified: virtual nodes -> shard_id)
    ring: RwLock<Vec<(u64, u32)>>,
    // Total number of virtual nodes
    validation_key: String,
}

impl ShardManager {
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            ring: RwLock::new(Vec::new()),
            validation_key: "shard_key".to_string(),
        }
    }

    pub async fn add_shard(&self, config: ShardConfig) -> anyhow::Result<()> {
        let pool = MySqlPool::connect_lazy(&config.database_url)?;

        let mut pools = self.pools.write().await;
        pools.insert(config.shard_id, pool);

        // Update ring
        let mut ring = self.ring.write().await;
        // Add virtual nodes
        let virtual_nodes = 100 * config.weight;
        for i in 0..virtual_nodes {
            let key = format!("{}:{}", config.shard_id, i);
            let hash = self.hash_key(&key);
            ring.push((hash, config.shard_id));
        }
        ring.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(())
    }

    pub async fn get_pool(&self, tenant_id: Uuid) -> Option<MySqlPool> {
        let shard_id = self.determine_shard_id(&tenant_id.to_string()).await?;
        let pools = self.pools.read().await;
        pools.get(&shard_id).cloned()
    }

    // Exposed for testing without DB connection
    pub async fn determine_shard_id(&self, key: &str) -> Option<u32> {
        let hash = self.hash_key(key);
        let ring = self.ring.read().await;
        if ring.is_empty() {
            return None;
        }

        // Find the first node with hash >= target hash
        match ring.binary_search_by(|(h, _)| h.cmp(&hash)) {
            Ok(idx) => Some(ring[idx].1),
            Err(idx) => {
                // If idx == len, wrap around to 0
                if idx == ring.len() {
                    Some(ring[0].1)
                } else {
                    Some(ring[idx].1)
                }
            }
        }
    }

    fn hash_key(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}
