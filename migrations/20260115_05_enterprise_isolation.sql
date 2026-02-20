-- Migration: Ensure Tenant Isolation and Indexes (Enterprise Grade)
-- Description: Adds missing tenant_id columns and composite indexes to enforce multi-tenancy.

-- Users: Ensure tenant_id is indexed
CREATE INDEX IF NOT EXISTS idx_users_tenant_email ON users(tenant_id, email);
CREATE INDEX IF NOT EXISTS idx_users_tenant_phone ON users(tenant_id, phone);

-- Roles: Ensure tenant_id is indexed (already in schema but double checking)
CREATE INDEX IF NOT EXISTS idx_roles_tenant_name ON roles(tenant_id, name);

-- Permissions: Ensure tenant_id is indexed (if permissions are tenant scoped)
-- Note: Permissions might be global system definitions or tenant specific.
-- For enterprise, custom permissions are often tenant scoped.
ALTER TABLE permissions ADD COLUMN IF NOT EXISTS tenant_id CHAR(36) DEFAULT NULL;
CREATE INDEX IF NOT EXISTS idx_permissions_tenant ON permissions(tenant_id);

-- Audit Logs: Ensure fast retrieval per tenant
CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant_created ON audit_logs(tenant_id, created_at DESC);

-- Sessions: Tenant isolation
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_user ON sessions(tenant_id, user_id);

-- Refresh Tokens: Tenant isolation
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_tenant_user ON refresh_tokens(tenant_id, user_id);

-- Clients (if exists, or future proofing for OAuth clients table)
CREATE TABLE IF NOT EXISTS oauth_clients (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    client_id VARCHAR(255) NOT NULL UNIQUE,
    client_secret VARCHAR(255) NOT NULL,
    redirect_uris TEXT NOT NULL, -- JSON array
    grant_types TEXT NOT NULL, -- JSON array
    scopes TEXT NOT NULL, -- JSON array
    name VARCHAR(255),
    logo_uri VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_oauth_clients_tenant (tenant_id),
    INDEX idx_oauth_clients_client_id (client_id)
);
