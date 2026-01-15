use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub mod config;
pub mod anomalies;

pub fn init_telemetry() -> anyhow::Result<()> {
    // 1. Setup Logging (Tracing)
    // We use a simple JSON formatter for structured logs
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().json());

    // 2. Setup OpenTelemetry (Optional / Mock for now)
    // In a real env, we'd add an OTLP exporter layer here.
    
    set_global_default(subscriber).map_err(|e| anyhow::anyhow!(e))?;

    // 3. Setup Metrics (Prometheus)
    // This starts a background HTTP listener on port 9000 by default or just prepares the recorder.
    // For this library, we'll install the recorder globally.
    let builder = PrometheusBuilder::new();
    builder.install().map_err(|e| anyhow::anyhow!(e))?;

    Ok(())
}
