use auth_audit::{AuditService, AuditLog};
use chrono::Utc;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let audit_service = AuditService::new(pool.clone());

    println!("Starting Audit System Test...");

    // 1. Create a series of logs
    let actor_id = Uuid::new_v4();
    println!("Actor ID: {}", actor_id);

    let log1 = audit_service.log(
        "TEST_ACTION_1",
        actor_id,
        "resource_a",
        Some(json!({"details": "first action"}))
    ).await?;
    println!("Created Log 1: {} (Hash: {})", log1.id, log1.hash);

    let log2 = audit_service.log(
        "TEST_ACTION_2",
        actor_id,
        "resource_b",
        None
    ).await?;
    println!("Created Log 2: {} (Hash: {})", log2.id, log2.hash);
    println!("Log 2 Prev Hash: {}", log2.prev_hash);

    // 2. Verify Chain Integrity (Simple check)
    assert_eq!(log2.prev_hash, log1.hash, "Chain broken: Log 2 prev_hash must match Log 1 hash");

    println!("Audit Chain Integrity Verified!");
    
    // 3. Verify Tamper Detection (Stub for now)
    // In a real test, we would update a row manually via SQL and assert verification fails.
    
    Ok(())
}
