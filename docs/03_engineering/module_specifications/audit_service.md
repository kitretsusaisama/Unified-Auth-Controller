---
title: Audit Service Specification
version: 1.0.0
status: Active
last_updated: 2026-01-12
owner: Engineering Team
category: Module Specification
crate: auth-audit
---

# Audit Service Specification

> [!NOTE]
> **Module**: `auth-audit`  
> **Responsibility**: Immutable audit logging and compliance exports

---

## 1. Overview

The **Audit Service** provides comprehensive, immutable audit logging for all security-critical events. It supports compliance requirements (SOC 2, HIPAA, PCI-DSS) with structured logging, PII masking, and export capabilities.

---

## 2. Public API

```rust
#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log_event(&self, event: AuditEvent) 
        -> Result<(), AuthError>;
    
    async fn query_logs(&self, filter: AuditFilter) 
        -> Result<Vec<AuditLogEntry>, AuthError>;
    
    async fn export_logs(&self, filter: AuditFilter, format: ExportFormat) 
        -> Result<Vec<u8>, AuthError>;
}
```

---

## 3. Models

### 3.1 Audit Event

```rust
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
}

pub enum AuditEventType {
    UserRegistered,
    UserLoggedIn,
    UserLoggedOut,
    UserLoginFailed,
    PasswordChanged,
    MfaEnabled,
    MfaDisabled,
    RoleAssigned,
    RoleRevoked,
    PermissionGranted,
    PermissionRevoked,
    TokenIssued,
    TokenRevoked,
    ConfigurationChanged,
}
```

### 3.2 Audit Log Entry

```rust
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
}
```

---

## 4. Event Types

### 4.1 Authentication Events

- `UserRegistered`: New user account created
- `UserLoggedIn`: Successful login
- `UserLoggedOut`: User logout
- `UserLoginFailed`: Failed login attempt
- `MfaEnabled`: MFA activated
- `MfaDisabled`: MFA deactivated

### 4.2 Authorization Events

- `RoleAssigned`: Role assigned to user
- `RoleRevoked`: Role removed from user
- `PermissionGranted`: Permission granted
- `PermissionRevoked`: Permission revoked

### 4.3 Token Events

- `TokenIssued`: Access/refresh token issued
- `TokenRevoked`: Token revoked

### 4.4 Configuration Events

- `ConfigurationChanged`: System configuration modified

---

## 5. Features

### 5.1 Immutability

**Implementation**:
- Append-only database table
- No UPDATE or DELETE operations
- Retention policy enforced at database level

**SQL**:
```sql
CREATE TABLE audit_logs (
    id BINARY(16) PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    event_type VARCHAR(50) NOT NULL,
    user_id BINARY(16),
    tenant_id BINARY(16) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    details JSON,
    INDEX idx_tenant_timestamp (tenant_id, timestamp),
    INDEX idx_user_timestamp (user_id, timestamp)
) ENGINE=InnoDB;
```

---

### 5.2 PII Masking

**Masked Fields**:
- Email addresses: `u***@example.com`
- IP addresses: `192.168.***.***`
- Phone numbers: `+1-***-***-1234`

**Implementation**:
```rust
pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        format!("{}***@{}", &local[..1], &domain[1..])
    } else {
        "***".to_string()
    }
}
```

---

### 5.3 Export Formats

**Supported Formats**:
- **CSV**: For Excel/spreadsheet analysis
- **JSON**: For programmatic processing
- **Parquet**: For big data analytics (planned)

**Example CSV Export**:
```csv
timestamp,event_type,user_id,tenant_id,ip_address,details
2026-01-12T10:00:00Z,UserLoggedIn,550e8400-...,tenant-id,192.168.1.1,"{""success"":true}"
```

---

## 6. Compliance

### 6.1 SOC 2 Requirements

- ✅ Immutable audit logs
- ✅ Timestamp accuracy
- ✅ User attribution
- ✅ Event details
- ✅ Retention policy (7 years)

### 6.2 HIPAA Requirements

- ✅ Access logging
- ✅ PII protection
- ✅ Audit trail integrity
- ✅ Export capabilities

### 6.3 PCI-DSS Requirements

- ✅ Authentication events
- ✅ Authorization events
- ✅ Log retention (1 year online, 3 years archive)

---

## 7. Performance

### 7.1 Write Performance

- **Target**: <10ms p95
- **Throughput**: 10k events/sec
- **Strategy**: Async writes, batching

### 7.2 Query Performance

- **Target**: <100ms p95 for filtered queries
- **Indexes**: tenant_id + timestamp, user_id + timestamp
- **Pagination**: Required for large result sets

---

## 8. Examples

### 8.1 Log Event

```rust
let event = AuditEvent {
    event_type: AuditEventType::UserLoggedIn,
    user_id: Some(user.id),
    tenant_id: tenant.id,
    ip_address: Some("192.168.1.1".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    details: json!({
        "success": true,
        "mfa_used": false,
    }),
};

audit_service.log_event(event).await?;
```

### 8.2 Query Logs

```rust
let filter = AuditFilter {
    tenant_id: Some(tenant.id),
    user_id: Some(user.id),
    event_types: vec![AuditEventType::UserLoggedIn],
    start_time: Some(Utc::now() - Duration::days(7)),
    end_time: Some(Utc::now()),
};

let logs = audit_service.query_logs(filter).await?;
```

### 8.3 Export Logs

```rust
let csv_data = audit_service.export_logs(filter, ExportFormat::CSV).await?;
std::fs::write("audit_export.csv", csv_data)?;
```

---

**Document Status**: Active  
**Owner**: Engineering Team
