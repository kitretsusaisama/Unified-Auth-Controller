-- Sprint 3: Lazy Registration
-- Migration: Add lazy registration configuration to tenants

ALTER TABLE tenants
ADD COLUMN allow_lazy_registration BOOLEAN DEFAULT FALSE NOT NULL,
ADD COLUMN lazy_registration_config JSON NULL;

-- Example config structure:
-- {
--   "required_fields": ["email", "full_name"],
--   "verification_method": "otp",
--   "default_role": "user"
-- }

-- Add status column to users if not fits lazy flow (we check UserStatus)
-- We might need a proper flag for "profile_incomplete" if PendingVerification isn't enough.
-- Adding is_profile_complete flag to users for progressive profiling
ALTER TABLE users
ADD COLUMN is_profile_complete BOOLEAN DEFAULT FALSE NOT NULL;

-- Index for finding incomplete profiles
CREATE INDEX idx_users_profile_complete ON users(is_profile_complete, tenant_id);

COMMENT ON COLUMN tenants.allow_lazy_registration IS 'Whether to allow automatic account creation on first login';
COMMENT ON COLUMN users.is_profile_complete IS 'Flag indicating if the user has completed their profile setup';
