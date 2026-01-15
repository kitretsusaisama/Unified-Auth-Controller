-- Migration: Create revoked_tokens table for access token blacklist
-- Part of Task 3.1: Implement JWT Token Engine with RS256
-- Requirements: 3.3, 3.5

-- Token revocation blacklist for immediate token invalidation
CREATE TABLE IF NOT EXISTS revoked_tokens (
    id CHAR(36) PRIMARY KEY,
    token_jti CHAR(36) UNIQUE NOT NULL,
    user_id CHAR(36) NOT NULL,
    tenant_id CHAR(36) NOT NULL,
    token_type ENUM('access', 'refresh') DEFAULT 'access',
    revoked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    revoked_by CHAR(36),
    revoked_reason VARCHAR(255),
    expires_at TIMESTAMP NOT NULL,
    
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    FOREIGN KEY (revoked_by) REFERENCES users(id) ON DELETE SET NULL,
    
    -- Indexes optimized for fast revocation checks
    INDEX idx_revoked_jti (token_jti),
    INDEX idx_revoked_user (user_id),
    INDEX idx_revoked_tenant (tenant_id),
    INDEX idx_revoked_expires (expires_at),
    INDEX idx_revoked_at (revoked_at),
    INDEX idx_revoked_type (token_type),
    
    -- Composite index for active revocation checks (most common query)
    INDEX idx_revoked_active (token_jti, expires_at)
);

-- Add comment for documentation
ALTER TABLE revoked_tokens COMMENT = 'Token revocation blacklist for immediate invalidation. Auto-cleaned after expiry.';
