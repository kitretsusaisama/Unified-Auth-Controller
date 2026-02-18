use crate::audit::{AuditLogger, AuditEvent};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{info, error};
use std::sync::Arc;

/// A channel-based audit logger that offloads writing to a background task
pub struct AsyncAuditLogger {
    sender: mpsc::Sender<AuditEvent>,
}

impl AsyncAuditLogger {
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<AuditEvent>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        (Self { sender: tx }, rx)
    }
}

#[async_trait]
impl AuditLogger for AsyncAuditLogger {
    async fn log(&self, event: AuditEvent) {
        // We use try_send to avoid blocking if buffer is full, or send to wait.
        // For audit, we generally prefer not to block the auth flow if logging is slow,
        // but we also don't want to lose logs.
        // A bounded channel provides backpressure.
        if let Err(e) = self.sender.send(event).await {
            error!("Failed to send audit event to background worker: {}", e);
        }
    }
}

/// The background worker that consumes events and writes them to the underlying storage
pub struct AuditWorker {
    receiver: mpsc::Receiver<AuditEvent>,
    delegate: Arc<dyn AuditLogger>,
}

impl AuditWorker {
    pub fn new(receiver: mpsc::Receiver<AuditEvent>, delegate: Arc<dyn AuditLogger>) -> Self {
        Self { receiver, delegate }
    }

    pub async fn run(mut self) {
        info!("Audit background worker started");
        while let Some(event) = self.receiver.recv().await {
            self.delegate.log(event).await;
        }
        info!("Audit background worker stopped");
    }
}
