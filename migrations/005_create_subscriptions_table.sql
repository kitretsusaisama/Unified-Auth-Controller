-- Tenant subscriptions and usage tracking
CREATE TABLE IF NOT EXISTS tenant_subscriptions (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    plan_id VARCHAR(50) NOT NULL,
    status ENUM('active', 'canceled', 'past_due', 'trialing') DEFAULT 'active',
    start_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_date TIMESTAMP NULL,
    current_usage JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    INDEX idx_sub_tenant (tenant_id),
    INDEX idx_sub_status (status)
);
