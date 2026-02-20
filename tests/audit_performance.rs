use auth_audit::AuditService;
use auth_db::connection::create_mysql_pool;
use serde_json::json;
use std::env;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;
use uuid::Uuid;

// Helper function to create test database config
fn create_test_database_config(url: &str) -> auth_config::DatabaseConfig {
    auth_config::DatabaseConfig {
        mysql_url: secrecy::Secret::new(url.to_string()),
        sqlite_url: None,
        max_connections: 50, // Higher connection count for concurrency test
        min_connections: 5,
        connection_timeout: 30,
        idle_timeout: 600,
        max_lifetime: 3600,
    }
}

#[tokio::test]
async fn benchmark_audit_log_throughput() {
    // This benchmark requires a running MySQL instance
    let result = env::var("TEST_MYSQL_URL");
    if result.is_err() {
        println!("Skipping audit benchmark - TEST_MYSQL_URL not set");
        return;
    }

    let database_url = result.unwrap();
    let pool = create_mysql_pool(&create_test_database_config(&database_url))
        .await
        .expect("Failed to create pool");

    let audit_service = Arc::new(AuditService::new(pool.clone()));

    let concurrency = 20;
    let iterations_per_task = 50;
    let total_ops = concurrency * iterations_per_task;

    println!("Starting benchmark: {} tasks * {} iterations = {} total inserts", concurrency, iterations_per_task, total_ops);

    let start = Instant::now();

    let mut handles = vec![];

    for _ in 0..concurrency {
        let service = audit_service.clone();
        let handle = task::spawn(async move {
            for _ in 0..iterations_per_task {
                let actor_id = Uuid::new_v4();
                let _ = service.log(
                    "benchmark_action",
                    actor_id,
                    "benchmark_resource",
                    Some(json!({"test": "data"}))
                ).await.expect("Log failed");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    let duration = start.elapsed();
    let seconds = duration.as_secs_f64();
    let ops_per_sec = total_ops as f64 / seconds;

    println!("Benchmark completed in {:.2}s", seconds);
    println!("Throughput: {:.2} logs/sec", ops_per_sec);
}
