-- Advanced RBAC/ABAC System Migration
-- Implements production-grade roles, permissions, and granular access control with strict tenant isolation

-- 1. Enhanced Permissions Table (Resource & Action based)
CREATE TABLE IF NOT EXISTS permissions (
    id CHAR(36) PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE, -- e.g., "user:create", "report:view"
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource_type VARCHAR(100) NOT NULL, -- e.g., "user", "report", "system"
    action VARCHAR(100) NOT NULL, -- e.g., "create", "read", "update", "delete", "approve"
    is_system_permission BOOLEAN DEFAULT FALSE, -- If true, cannot be deleted
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_perm_code (code),
    INDEX idx_perm_resource (resource_type)
);

-- 2. Enhanced Roles Table (Tenant & Org Scope)
-- Note: 'roles' table exists in 001, we ALTER it to ensure compliance or CREATE if missing features
-- We will DROP and RECREATE to ensure clean state given the major architectural shift or ALTER safely.
-- For safety in this "Hyper Advanced" mode, we'll ALTER.

-- Ensure existing roles table has necessary columns for ABAC
ALTER TABLE roles
    ADD COLUMN IF NOT EXISTS organization_id CHAR(36) NULL,
    ADD COLUMN IF NOT EXISTS scope ENUM('global', 'organization', 'tenant') DEFAULT 'tenant',
    ADD COLUMN IF NOT EXISTS metadata JSON NULL, -- For UI/display preferences
    ADD COLUMN IF NOT EXISTS version INT DEFAULT 1;

CREATE INDEX idx_role_scope ON roles(scope);
CREATE INDEX idx_role_org ON roles(organization_id);

-- 3. Role-Permissions Mapping (Many-to-Many)
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id CHAR(36) NOT NULL,
    permission_id CHAR(36) NOT NULL,
    conditions JSON NULL, -- ABAC Conditions (e.g., {"resource_owner": true})
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

-- 4. User-Role Assignment Audit (Enhanced)
-- user_roles exists, we ensure it has audit fields
ALTER TABLE user_roles
    ADD COLUMN IF NOT EXISTS assignment_context JSON NULL, -- Context of assignment (e.g., via Group Sync)
    ADD COLUMN IF NOT EXISTS version INT DEFAULT 1;

-- 5. Audit Log Table for Authorization Changes (Dedicated)
CREATE TABLE IF NOT EXISTS authorization_audit_logs (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36),
    actor_id CHAR(36) NOT NULL,
    action VARCHAR(100) NOT NULL, -- "assign_role", "revoke_role", "create_role"
    target_type VARCHAR(50) NOT NULL, -- "user", "role", "permission"
    target_id CHAR(36) NOT NULL,
    changes JSON NOT NULL, -- Before/After state
    ip_address VARCHAR(45),
    user_agent VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_auth_audit_tenant (tenant_id),
    INDEX idx_auth_audit_actor (actor_id),
    INDEX idx_auth_audit_target (target_type, target_id)
);

-- 6. Seed Basic System Permissions (Idempotent)
INSERT IGNORE INTO permissions (id, code, name, resource_type, action, is_system_permission) VALUES
('perm_sys_admin', 'system:admin', 'System Administrator', 'system', 'admin', TRUE),
('perm_tenant_manage', 'tenant:manage', 'Manage Tenant', 'tenant', 'manage', TRUE),
('perm_user_read', 'user:read', 'View Users', 'user', 'read', TRUE),
('perm_user_write', 'user:write', 'Edit Users', 'user', 'write', TRUE),
('perm_role_manage', 'role:manage', 'Manage Roles', 'role', 'manage', TRUE);
