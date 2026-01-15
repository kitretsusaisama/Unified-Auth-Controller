-- Migration: Create refresh_tokens table with family tracking
-- Part of Task 3.3: Implement Refresh Token System with Family Tracking
-- Requirements: 3.4, 7.1

-- Enhanced refresh tokens with family tracking for security
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    token_family CHAR(36) NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    device_fingerprint VARCHAR(255),
    user_agent TEXT,
    ip_address VARCHAR(45),
    expires_at TIMESTAMP NOT NULL,
    revoked_at TIMESTAMP NULL,
    revoked_reason VARCHAR(100) NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Indexes for fast lookup and family tracking
    INDEX idx_rt_token_hash (token_hash),
    INDEX idx_rt_user (user_id),
    INDEX idx_rt_tenant (tenant_id),
    INDEX idx_rt_family (token_family),
    INDEX idx_rt_expires (expires_at),
    INDEX idx_rt_active (revoked_at, expires_at),
    INDEX idx_rt_user_tenant (user_id, tenant_id),
    INDEX idx_rt_created (created_at)
);

-- Add comment for documentation
ALTER TABLE refresh_tokens COMMENT = 'Stores refresh tokens with family tracking for breach detection';
