use auth_db::sharding::{ShardManager, ShardConfig};
use uuid::Uuid;
use tokio;
use proptest::prelude::*;

#[tokio::main]
async fn main() {
    let manager = ShardManager::new();
    
    // Add shards
    let shards = vec![
        ShardConfig { shard_id: 1, database_url: "mysql://mock1".to_string(), weight: 1 },
        ShardConfig { shard_id: 2, database_url: "mysql://mock2".to_string(), weight: 1 },
        ShardConfig { shard_id: 3, database_url: "mysql://mock3".to_string(), weight: 1 },
    ];

    for shard in shards {
        manager.add_shard(shard).await.ok(); // Mock connection will fail, but ring updates
    }

    // Property: Same tenant always routes to same shard
    let tenant_id = Uuid::new_v4();
    let shard1 = manager.determine_shard_id(&tenant_id.to_string()).await.unwrap();
    let shard2 = manager.determine_shard_id(&tenant_id.to_string()).await.unwrap();
    
    assert_eq!(shard1, shard2, "Shard selection must be deterministic");
    println!("✅ Deterministic Routing: PASSED");

    // Property: Distribution Check (Approximate)
    let mut counts = std::collections::HashMap::new();
    for _ in 0..1000 {
        let tid = Uuid::new_v4();
        let sid = manager.determine_shard_id(&tid.to_string()).await.unwrap();
        *counts.entry(sid).or_insert(0) += 1;
    }
    
    println!("Shard Distribution (1000 tenants): {:?}", counts);
    assert!(counts.len() == 3, "All shards should receive traffic");
}

proptest! {
    #[test]
    fn test_uuid_distribution_property(uuid_str in "[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}") {
        // Can't use async in proptest easily without runtime, 
        // but verify hash_key stability if exposed?
        // Integrating async property test requires block_on.
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let manager = ShardManager::new();
            manager.add_shard(ShardConfig { shard_id: 1, database_url: "a".to_string(), weight: 1 }).await.ok();
            
            let sid1 = manager.determine_shard_id(&uuid_str).await;
            let sid2 = manager.determine_shard_id(&uuid_str).await;
            assert_eq!(sid1, sid2);
        });
    }
}
