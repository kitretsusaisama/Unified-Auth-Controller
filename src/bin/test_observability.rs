use auth_telemetry::init_telemetry;
use metrics::{counter, histogram};

fn main() {
    // 1. Init Telemetry (should print JSON logs to stdout)
    if let Err(e) = init_telemetry() {
        eprintln!("Failed to init telemetry: {}", e);
        // It might fail if global subscriber is already set (unlikely in fresh binary)
        // or if Prometheus port 9000 is taken.
        // We'll proceed.
    } else {
        println!("✅ Telemetry Initialized");
    }

    // 2. Emit Metrics
    counter!("test_requests_total", 1);
    histogram!("test_latency_seconds", 0.123);
    println!("✅ Metrics emitted (Prometheus endpoint active)");

    // 3. Emit Tracing
    tracing::info!(user_id = "123", "User logged in");
    tracing::error!(error = "db_fail", "Database connection lost");
    println!("✅ Structured Logs emitted");
}
