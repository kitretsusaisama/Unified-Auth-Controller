# Database Schema

## Tables

### Users (`users`)
- `id`: UUID (PK)
- `tenant_id`: UUID (Indexed)
- `email`: VARCHAR (Unique per tenant)
- `password_hash`: VARCHAR
- `status`: ENUM ('active', 'suspended')

### Roles (`roles`)
- `id`: UUID
- `tenant_id`: UUID
- `name`: VARCHAR
- `permissions`: JSON

### Refresh Tokens (`refresh_tokens`)
- `id`: UUID
- `token_hash`: VARCHAR (Indexed)
- `family_id`: UUID (Rotation)
- `tenant_id`: UUID

### Audit Logs (`audit_logs`)
- `id`: UUID
- `tenant_id`: UUID
- `event_type`: VARCHAR
- `payload`: JSON
- `created_at`: TIMESTAMP

## Tenant Isolation
All tables (except global system configs) include `tenant_id`. Indexes are composite `(tenant_id, ...)` to ensure performance and isolation.
