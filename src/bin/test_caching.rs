use auth_cache::{Cache, MultiLevelCache};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TestUser {
    id: u32,
    name: String,
}

#[tokio::main]
async fn main() {
    let redis_url = "redis://127.0.0.1/";

    let cache = match MultiLevelCache::new(redis_url) {
        Ok(c) => c,
        Err(e) => {
            println!("⚠️ Redis connection failed: {}", e);
            // We can still test L1 if we bypass constructor? No, constructor needs client.
            // But if new() fails, we just exit.
            return;
        }
    };

    println!("Connected to Cache");

    let user = TestUser {
        id: 1,
        name: "Vic".to_string(),
    };

    // Set
    cache
        .set("user:1", &user, Duration::from_secs(10))
        .await
        .expect("Set failed");

    // Get L1
    // Explicit type to satisfy E0282
    let fetched = cache
        .get::<TestUser>("user:1")
        .await
        .expect("Get failed")
        .expect("Key missing");
    assert_eq!(user, fetched);
    println!("✅ L1 Get Passed");

    // Delete
    cache.delete("user:1").await.expect("Delete failed");

    let missing = cache.get::<TestUser>("user:1").await.expect("Get failed");
    assert!(missing.is_none());
    println!("✅ Delete Passed");
}
