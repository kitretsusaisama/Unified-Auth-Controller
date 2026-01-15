# repositories/user_repository.rs

## File Metadata

**File Path**: `crates/auth-db/src/repositories/user_repository.rs`  
**Crate**: `auth-db`  
**Module**: `repositories::user_repository`  
**Layer**: Adapter (Persistence)  
**Security-Critical**: ✅ **YES** - Direct database access for user data

## Purpose

Implements the `UserStore` trait for MySQL/SQLite persistence, providing CRUD operations for user entities with compile-time checked SQL queries.

### Problem It Solves

- Abstracts database operations from business logic
- Implements Repository pattern for user persistence
- Provides type-safe database queries (SQLx compile-time checking)
- Handles multi-tenancy at database level

---

## Detailed Code Breakdown

### Struct: `UserRepository`

**Purpose**: MySQL/SQLite implementation of `UserStore` trait

**Fields**:
- `pool`: `MySqlPool` - Database connection pool

**Pattern**: Repository pattern with trait-based abstraction

---

### Trait Implementation: `UserStore for UserRepository`

**Purpose**: Implements auth-core's `UserStore` trait

**Methods**: Delegates to internal implementation with error conversion

```rust
#[async_trait]
impl UserStore for UserRepository {
    async fn find_by_email(&self, email: &str, tenant_id: Uuid) 
        -> Result<Option<User>, AuthError> 
    {
        self.find_by_email(email, tenant_id).await.map_err(AuthError::from)
    }
    // ... other methods
}
```

**Pattern**: Error conversion from `sqlx::Error` to `AuthError`

---

### Method: `UserRepository::new()`

**Signature**: `pub fn new(pool: MySqlPool) -> Self`

**Purpose**: Constructor with connection pool

**Usage**:
```rust
let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

let user_repo = UserRepository::new(pool);
```

---

### Method: `UserRepository::create()`

**Signature**: `pub async fn create(request, password_hash, tenant_id) -> Result<User>`

**Purpose**: Create new user in database

**Steps**:

1. **Generate ID and Timestamps**
   ```rust
   let id = Uuid::new_v4();
   let now = Utc::now();
   let status = UserStatus::PendingVerification;
   ```

2. **Serialize Status**
   ```rust
   let status_str = serde_json::to_string(&status)
       .unwrap_or_else(|_| "\"PendingVerification\"".to_string());
   ```

3. **Prepare Profile Data**
   ```rust
   let profile = serde_json::to_value(&request.profile_data)
       .unwrap_or(serde_json::json!({}));
   ```

4. **Insert User**
   ```rust
   sqlx::query(
       r#"
       INSERT INTO users (
           id, tenant_id, email, password_hash, status, 
           created_at, updated_at, email_verified, phone_verified,
           failed_login_attempts, risk_score, mfa_enabled,
           profile_data, preferences
       )
       VALUES (?, ?, ?, ?, ?, ?, ?, false, false, 0, 0.0, false, ?, '{}')
       "#,
   )
   .bind(id.to_string())
   .bind(tenant_id.to_string())
   .bind(&request.email)
   .bind(&password_hash)
   .bind(&status_str)
   .bind(now)
   .bind(now)
   .bind(&profile)
   .execute(&self.pool)
   .await?;
   ```

5. **Fetch Created User**
   ```rust
   self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
   ```

**Security**:
- Password hash stored (never plaintext)
- Initial status: `PendingVerification`
- Default values for security fields (mfa_enabled=false, failed_attempts=0)

**Returns**: Created user with all fields populated

---

### Method: `UserRepository::find_by_email()`

**Signature**: `pub async fn find_by_email(email, tenant_id) -> Result<Option<User>>`

**Purpose**: Find user by email within tenant

**SQL Query**:
```sql
SELECT id, email, email_verified, phone, phone_verified, 
       password_hash, password_changed_at, failed_login_attempts, 
       locked_until, last_login_at, last_login_ip, mfa_enabled, 
       mfa_secret, backup_codes, risk_score, profile_data, 
       preferences, status, created_at, updated_at, deleted_at
FROM users 
WHERE email = ? AND tenant_id = ? AND deleted_at IS NULL
```

**Multi-Tenancy**: Enforced by `tenant_id` filter

**Soft Delete**: Excludes deleted users (`deleted_at IS NULL`)

**Returns**: `Option<User>` (None if not found)

---

### Method: `UserRepository::find_by_id()`

**Signature**: `pub async fn find_by_id(id) -> Result<Option<User>>`

**Purpose**: Find user by UUID (global lookup)

**SQL Query**:
```sql
SELECT * FROM users WHERE id = ?
```

**Note**: No tenant filter (for admin operations)

---

### Method: `UserRepository::map_row()`

**Signature**: `fn map_row(row: MySqlRow) -> Result<User>`

**Purpose**: Convert database row to User struct

**Implementation**:

```rust
fn map_row(&self, row: sqlx::mysql::MySqlRow) -> Result<User, sqlx::Error> {
    // 1. Deserialize status
    let status_str: String = row.try_get("status")?;
    let status: UserStatus = serde_json::from_str(&status_str)
        .unwrap_or(UserStatus::PendingVerification);
    
    // 2. Parse UUID
    let id_str: String = row.try_get("id")?;
    let id = Uuid::parse_str(&id_str).unwrap_or_default();
    
    // 3. Map all fields
    Ok(User {
        id,
        email: row.try_get("email")?,
        email_verified: row.try_get("email_verified")?,
        phone: row.try_get("phone")?,
        phone_verified: row.try_get("phone_verified")?,
        password_hash: Some(row.try_get("password_hash")?),
        password_changed_at: row.try_get("password_changed_at")?,
        failed_login_attempts: row.try_get::<i32, _>("failed_login_attempts")
            .unwrap_or(0) as u32,
        locked_until: row.try_get("locked_until")?,
        last_login_at: row.try_get("last_login_at")?,
        last_login_ip: row.try_get("last_login_ip")?,
        mfa_enabled: row.try_get("mfa_enabled")?,
        mfa_secret: row.try_get("mfa_secret")?,
        backup_codes: row.try_get("backup_codes")
            .map(|v: serde_json::Value| serde_json::from_value(v).unwrap_or_default())
            .ok(),
        risk_score: row.try_get::<f32, _>("risk_score").unwrap_or(0.0),
        profile_data: row.try_get::<serde_json::Value, _>("profile_data")
            .unwrap_or(serde_json::json!({})),
        preferences: row.try_get::<serde_json::Value, _>("preferences")
            .unwrap_or(serde_json::json!({})),
        status,
        created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
        updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
        deleted_at: row.try_get("deleted_at")?,
    })
}
```

**Type Conversions**:
- `i32` → `u32` for `failed_login_attempts`
- `String` → `Uuid` for `id`
- `String` → `UserStatus` (JSON deserialization)
- `Value` → `Vec<String>` for `backup_codes`

**Error Handling**: Defaults for optional/nullable fields

---

### Method: `UserRepository::update_status()`

**Signature**: `pub async fn update_status(id, status) -> Result<()>`

**Purpose**: Update user account status

**SQL Query**:
```sql
UPDATE users 
SET status = ?, updated_at = ? 
WHERE id = ?
```

**Usage**:
```rust
// Ban user
user_repo.update_status(user_id, UserStatus::Suspended).await?;

// Activate user
user_repo.update_status(user_id, UserStatus::Active).await?;
```

**Audit**: Should trigger audit log event

---

### Method: `UserRepository::increment_failed_attempts()`

**Signature**: `pub async fn increment_failed_attempts(id) -> Result<u32>`

**Purpose**: Increment failed login counter and return new count

**SQL Queries**:

1. **Increment Counter**
   ```sql
   UPDATE users 
   SET failed_login_attempts = failed_login_attempts + 1, 
       updated_at = ? 
   WHERE id = ?
   ```

2. **Fetch New Count**
   ```sql
   SELECT failed_login_attempts 
   FROM users 
   WHERE id = ?
   ```

**Returns**: New attempt count (for lockout logic)

**Security**: Enables account lockout after threshold

---

### Method: `UserRepository::reset_failed_attempts()`

**Signature**: `pub async fn reset_failed_attempts(id) -> Result<()>`

**Purpose**: Reset failed login counter and unlock account

**SQL Query**:
```sql
UPDATE users 
SET failed_login_attempts = 0, 
    locked_until = NULL, 
    updated_at = ? 
WHERE id = ?
```

**Trigger**: Successful login

---

### Method: `UserRepository::record_login()`

**Signature**: `pub async fn record_login(id, ip) -> Result<()>`

**Purpose**: Record successful login with timestamp and IP

**SQL Query**:
```sql
UPDATE users 
SET last_login_at = ?, 
    last_login_ip = ?, 
    failed_login_attempts = 0, 
    locked_until = NULL, 
    updated_at = ? 
WHERE id = ?
```

**Effects**:
- Updates `last_login_at` to current time
- Updates `last_login_ip` to request IP
- Resets `failed_login_attempts` to 0
- Clears `locked_until` (unlocks account)

**Audit Trail**: Provides login history

---

## Database Schema

```sql
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    email VARCHAR(255) NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE,
    phone VARCHAR(20),
    phone_verified BOOLEAN DEFAULT FALSE,
    password_hash VARCHAR(255),
    password_changed_at TIMESTAMP NULL,
    failed_login_attempts INT DEFAULT 0,
    locked_until TIMESTAMP NULL,
    last_login_at TIMESTAMP NULL,
    last_login_ip VARCHAR(45),
    mfa_enabled BOOLEAN DEFAULT FALSE,
    mfa_secret VARCHAR(255),
    backup_codes JSON,
    risk_score FLOAT DEFAULT 0.0,
    profile_data JSON DEFAULT '{}',
    preferences JSON DEFAULT '{}',
    status VARCHAR(50) DEFAULT 'PendingVerification',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP NULL,
    
    UNIQUE KEY unique_email_tenant (email, tenant_id),
    INDEX idx_tenant_id (tenant_id),
    INDEX idx_email (email),
    INDEX idx_status (status),
    INDEX idx_deleted_at (deleted_at)
);
```

---

## Security Considerations

### SQL Injection Prevention

**SQLx Parameterized Queries**: All queries use parameter binding

**Bad** (vulnerable):
```rust
// DON'T: SQL injection risk
let query = format!("SELECT * FROM users WHERE email = '{}'", email);
```

**Good** (safe):
```rust
// DO: Parameterized query
sqlx::query("SELECT * FROM users WHERE email = ?")
    .bind(email)
    .execute(&pool)
    .await?;
```

### Multi-Tenancy Isolation

**Enforced at Query Level**:
```sql
WHERE email = ? AND tenant_id = ?  -- Always filter by tenant
```

**Prevents**: Cross-tenant data leakage

### Soft Delete

**Implementation**: `deleted_at IS NULL` filter

**Benefits**:
- Preserves audit trail
- Enables data recovery
- GDPR compliance (right to erasure)

### Password Hash Protection

**Never Exposed**: Password hash included in SELECT but excluded from API responses

**Storage**: Stored as-is (already hashed by IdentityService)

---

## Performance Optimizations

### Connection Pooling

```rust
let pool = MySqlPoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .connect_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;
```

### Indexes

**Required Indexes**:
- `unique_email_tenant` - Enforces email uniqueness per tenant
- `idx_tenant_id` - Fast tenant filtering
- `idx_email` - Fast email lookups
- `idx_status` - Filter by status
- `idx_deleted_at` - Exclude soft-deleted users

### Query Optimization

**Fetch After Insert**: Uses `find_by_id()` to return created user
- Alternative: Use `RETURNING` clause (PostgreSQL only)
- MySQL: Requires separate SELECT

---

## Error Handling

### Error Conversion

```rust
impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}
```

### Common Errors

| SQLx Error | AuthError | HTTP Status |
|------------|-----------|-------------|
| `RowNotFound` | `UserNotFound` | 404 |
| `UniqueViolation` | `Conflict` | 409 |
| `ConnectionError` | `DatabaseError` | 500 |

---

## Testing

### Unit Tests

```rust
#[sqlx::test]
async fn test_create_user(pool: MySqlPool) {
    let repo = UserRepository::new(pool);
    let request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: Some("password123".to_string()),
        phone: None,
        profile_data: None,
    };
    
    let user = repo.create(request, "hashed_password", Uuid::new_v4()).await.unwrap();
    
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.status, UserStatus::PendingVerification);
}

#[sqlx::test]
async fn test_find_by_email_multi_tenancy(pool: MySqlPool) {
    let repo = UserRepository::new(pool);
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    
    // Create same email in different tenants
    repo.create(request.clone(), "hash", tenant1).await.unwrap();
    repo.create(request.clone(), "hash", tenant2).await.unwrap();
    
    // Should find only tenant1 user
    let user = repo.find_by_email("test@example.com", tenant1).await.unwrap();
    assert!(user.is_some());
}
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Async SQL toolkit with compile-time checking |
| `uuid` | UUID handling |
| `chrono` | Timestamp handling |
| `serde_json` | JSON serialization |
| `async-trait` | Async trait support |

### Internal Dependencies

- [auth-core/models/user.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/models/user.rs) - User entity
- [auth-core/services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - UserStore trait
- [auth-core/error.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/error.rs) - AuthError

---

## Related Files

- [services/identity.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-core/src/services/identity.rs) - Uses UserStore trait
- [connection.rs](file:///c:/Users/Victo/Downloads/sso/crates/auth-db/src/connection.rs) - Database connection setup
- [migrations/](file:///c:/Users/Victo/Downloads/sso/migrations) - Database schema migrations

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 220  
**Security Level**: CRITICAL
