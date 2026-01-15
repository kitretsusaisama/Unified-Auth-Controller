CREATE TABLE audit_logs (
    id CHAR(36) PRIMARY KEY NOT NULL,
    action VARCHAR(64) NOT NULL,
    actor_id CHAR(36) NOT NULL,
    resource VARCHAR(255) NOT NULL,
    metadata JSON,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    hash VARCHAR(64) NOT NULL,
    prev_hash VARCHAR(64) NOT NULL,
    
    INDEX idx_audit_actor (actor_id),
    INDEX idx_audit_action (action),
    INDEX idx_audit_timestamp (timestamp)
);
