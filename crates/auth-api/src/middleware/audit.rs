use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use auth_core::audit::{AuditLogger, AuditEvent, AuditCategory, AuditSeverity, AuditOutcome};

pub async fn audit_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    let user_agent = req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Get audit logger from extensions
    let audit_logger = match req.extensions().get::<Arc<dyn AuditLogger>>() {
        Some(logger) => logger.clone(),
        None => {
            // If no audit logger is available, create a dummy response
            let response = next.run(req).await;
            return response;
        }
    };
    
    // TODO: Extract IP (needs SecureClientIp source or similar)
    let ip_address: Option<String> = None; 
    
    // Run the handler
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    // Determine severity/outcome based on status
    let (severity, outcome) = if status.is_server_error() {
        (AuditSeverity::Critical, AuditOutcome::Failure { reason: status.to_string() })
    } else if status.is_client_error() {
        (AuditSeverity::Warning, AuditOutcome::Failure { reason: status.to_string() })
    } else {
        (AuditSeverity::Info, AuditOutcome::Success)
    };
    
    // Create event
    // Note: To get Actor ID (User ID), we would need to extract it from Extensions populated by Auth middleware.
    // Assuming we might have `Extension<Claims>` or similar later.
    let event = AuditEvent::new(
        AuditCategory::System, // Default to System for raw HTTP logs
        format!("HTTP {} {}", method, uri.path()),
        severity,
    )
    .with_context(ip_address, user_agent, None)
    .with_metadata(serde_json::json!({
        "method": method.to_string(),
        "path": uri.path(),
        "query": uri.query(),
        "status": status.as_u16(),
        "duration_ms": duration.as_millis()
    }));
    
    // We modify event outcome
    let event = match outcome {
        AuditOutcome::Success => event,
        AuditOutcome::Failure { reason } => event.failure(reason),
    };

    // Spawn log task so we don't block response
    // (AuditLogger::log is async, but we can spawn it)
    let logger = audit_logger.clone();
    tokio::spawn(async move {
        logger.log(event).await;
    });

    response
}
