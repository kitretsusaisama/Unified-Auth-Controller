use auth_telemetry::anomalies::detector::AnomalyDetector;

fn main() {
    let detector = AnomalyDetector::new(20, 2.0); // Window=20, Threshold=2.0 (Sigma)

    println!("Starting Anomaly Detection Test...");

    // 1. Train with normal data (Mean ~10, Variance minimal)
    for i in 0..20 {
        let val = 10.0 + (rand::random::<f64>() - 0.5); // 9.5 to 10.5
        let is_anomaly = detector.record("ip:1.2.3.4", val);
        if is_anomaly {
            println!("❌ False positive at step {}: val={}", i, val);
        }
        assert!(!is_anomaly, "Normal data should not be anomalous");
    }
    println!("✅ Training Phase (Normal Traffic) Passed");

    // 2. Inject Anomaly (Spike to 50)
    let val = 50.0;
    let is_anomaly = detector.record("ip:1.2.3.4", val);
    if !is_anomaly {
        println!("❌ False negative: val={}", val);
    }
    assert!(
        is_anomaly,
        "Spike (50.0) should be detected as anomaly vs Mean ~10.0"
    );
    println!("✅ Anomaly Detection (Spike) Passed");

    // 3. Return to normal
    let is_anomaly_return = detector.record("ip:1.2.3.4", 10.0);
    assert!(
        !is_anomaly_return,
        "Return to normal should not be anomalous"
    );
    println!("✅ Recovery Phase Passed");
}
