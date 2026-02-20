use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::warn;

// Simple statistical anomaly detector using Z-Score approximation (Rolling Mean/Variance)
pub struct AnomalyDetector {
    // Key (e.g., "login_failure:IP") -> Window of values
    windows: Arc<Mutex<HashMap<String, VecDeque<f64>>>>,
    window_size: usize,
    threshold: f64, // Standard Deviations
}

impl AnomalyDetector {
    pub fn new(window_size: usize, threshold: f64) -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            window_size,
            threshold,
        }
    }

    pub fn record(&self, key: &str, value: f64) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let window = windows.entry(key.to_string()).or_default();

        // Calculate stats on CURRENT window
        let is_anomaly = if window.len() >= 10 {
            // Min samples
            let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
            let variance: f64 =
                window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / window.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev > 0.0 && (value - mean).abs() > (self.threshold * std_dev) {
                warn!(
                    "Anomaly Detected for {}: Value={}, Mean={}, StdDev={}",
                    key, value, mean, std_dev
                );
                true
            } else {
                false
            }
        } else {
            false
        };

        // Update Window
        if window.len() >= self.window_size {
            window.pop_front();
        }
        window.push_back(value);

        is_anomaly
    }
}
