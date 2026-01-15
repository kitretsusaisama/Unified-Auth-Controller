
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub jaeger_endpoint: Option<String>,
}
