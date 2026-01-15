-- Sprint 1: Multi-Channel Authentication Foundation
-- Migration: Add support for email OR phone as primary identifier

-- Add new columns to users table
ALTER TABLE users
ADD COLUMN identifier_type VARCHAR(10) DEFAULT 'email' NOT NULL,
ADD COLUMN phone VARCHAR(20) NULL,
ADD COLUMN primary_identifier VARCHAR(10) DEFAULT 'email' NOT NULL,
ADD COLUMN phone_verified BOOLEAN DEFAULT FALSE NOT NULL,
ADD COLUMN phone_verified_at TIMESTAMP NULL;

-- Update existing constraints
-- Make email nullable (since phone can be primary identifier)
ALTER TABLE users 
MODIFY COLUMN email VARCHAR(255) NULL;

-- Add check constraint for identifier_type
ALTER TABLE users
ADD CONSTRAINT chk_identifier_type 
CHECK (identifier_type IN ('email', 'phone', 'both'));

-- Add check constraint for primary_identifier
ALTER TABLE users
ADD CONSTRAINT chk_primary_identifier 
CHECK (primary_identifier IN ('email', 'phone'));

-- Ensure at least one identifier exists
ALTER TABLE users
ADD CONSTRAINT chk_has_identifier
CHECK (email IS NOT NULL OR phone IS NOT NULL);

-- Add unique constraint for phone per tenant
CREATE UNIQUE INDEX idx_users_phone_tenant 
ON users(phone, tenant_id) 
WHERE phone IS NOT NULL;

-- Update existing unique constraint for email to be conditional
DROP INDEX idx_users_email_tenant;
CREATE UNIQUE INDEX idx_users_email_tenant 
ON users(email, tenant_id) 
WHERE email IS NOT NULL;

-- Add index for identifier lookups
CREATE INDEX idx_users_identifier_type ON users(identifier_type);
CREATE INDEX idx_users_primary_identifier ON users(primary_identifier);
CREATE INDEX idx_users_phone ON users(phone) WHERE phone IS NOT NULL;

-- Add index for phone verification status
CREATE INDEX idx_users_phone_verified ON users(phone_verified, phone_verified_at);

-- Comments for documentation
COMMENT ON COLUMN users.identifier_type IS 'Type of identifier(s) used: email, phone, or both';
COMMENT ON COLUMN users.phone IS 'User phone number in E.164 format (e.g., +14155552671)';
COMMENT ON COLUMN users.primary_identifier IS 'Primary identifier for login: email or phone';
COMMENT ON COLUMN users.phone_verified IS 'Whether phone number has been verified';
COMMENT ON COLUMN users.phone_verified_at IS 'Timestamp when phone was verified';

-- Migrate existing users to new schema
UPDATE users 
SET identifier_type = 'email',
    primary_identifier = 'email'
WHERE identifier_type IS NULL;
