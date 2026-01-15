# main.rs

## File Metadata

**File Path**: `src/main.rs`  
**Crate**: Root  
**Module**: Main  
**Layer**: Application Entry Point  
**Security-Critical**: ⚠️ **MEDIUM** - Application bootstrap

## Purpose

Main application entry point that initializes and runs the authentication server.

### Problem It Solves

- Application bootstrap
- Server initialization
- Configuration loading
- Graceful shutdown

---

## Typical Implementation

```rust
use auth_api::api_router;
use auth_telemetry::init_telemetry;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    init_telemetry()?;
    
    // Load configuration
    let config = auth_config::load_config()?;
    
    // Create router
    let app = api_router();
    
    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Security Level**: MEDIUM
