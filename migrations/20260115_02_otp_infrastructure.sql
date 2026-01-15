-- Sprint 2: OTP Infrastructure
-- Migration: Create OTP sessions table for secure OTP storage

CREATE TABLE otp_sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NULL,  -- Nullable for new user registration
    tenant_id VARCHAR(36) NOT NULL,
    
    -- Identifier information
    identifier_type VARCHAR(10) NOT NULL,
    identifier VARCHAR(255) NOT NULL,  -- Email or phone
    
    -- OTP data
    otp_hash VARCHAR(255) NOT NULL,  -- Hashed OTP for security
    attempts INT DEFAULT 0 NOT NULL,
    max_attempts INT DEFAULT 5 NOT NULL,
    
    -- Delivery tracking
    delivery_method VARCHAR(10) NOT NULL,  -- 'email' or 'sms'
    sent_at TIMESTAMP NOT NULL,
    
    -- Session management
    expires_at TIMESTAMP NOT NULL,
    verified_at TIMESTAMP NULL,
    
    -- Purpose tracking
    purpose VARCHAR(50) NOT NULL,  -- 'registration', 'login', 'verification', 'password_reset'
    
    -- Metadata
    ip_address VARCHAR(45) NULL,
    user_agent TEXT NULL,
    
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    
    -- Indexes
    INDEX idx_otp_identifier (identifier, tenant_id),
    INDEX idx_otp_user (user_id, tenant_id),
    INDEX idx_otp_expires (expires_at),
    INDEX idx_otp_purpose (purpose),
    
    -- Constraints
    CHECK (identifier_type IN ('email', 'phone')),
    CHECK (delivery_method IN ('email', 'sms')),
    CHECK (purpose IN ('registration', 'login', 'verification', 'password_reset')),
    CHECK (attempts <= max_attempts),
    
    -- Foreign keys
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

-- Add index for cleanup job
CREATE INDEX idx_otp_cleanup ON otp_sessions(expires_at, verified_at);

-- Comments
COMMENT ON TABLE otp_sessions IS 'Stores OTP sessions for multi-factor authentication and verification';
COMMENT ON COLUMN otp_sessions.otp_hash IS 'Hashed OTP using bcrypt - never store plain OTP';
COMMENT ON COLUMN otp_sessions.attempts IS 'Number of verification attempts made';
COMMENT ON COLUMN otp_sessions.purpose IS 'Purpose of OTP: registration, login, verification, or password reset';
