# connection.rs

## File Metadata

**File Path**: `crates/auth-db/src/connection.rs`  
**Crate**: `auth-db`  
**Module**: `connection`  
**Layer**: Infrastructure (Database)  
**Security-Critical**: ⚠️ **MEDIUM** - Database connection management

## Purpose

Manages database connection pooling for MySQL and SQLite, providing secure connection establishment with credential protection.

### Problem It Solves

- Database connection pooling
- Multi-database support (MySQL, SQLite)
- Secure credential handling
- Connection lifecycle management

---

## Detailed Code Breakdown

### Enum: `DatabasePool`

**Purpose**: Abstract over different database types

**Variants**:
- `MySql(MySqlPool)` - MySQL connection pool
- `Sqlite(SqlitePool)` - SQLite connection pool

---

### Function: `create_mysql_pool()`

**Signature**: `pub async fn create_mysql_pool(config: &DatabaseConfig) -> Result<Pool<MySql>>`

**Purpose**: Create MySQL connection pool

**Process**:
```rust
let pool = MySqlPool::connect(config.mysql_url.expose_secret()).await?;
Ok(pool)
```

**Security**: Uses `secrecy::ExposeSecret` to safely access credentials

---

### Function: `create_sqlite_pool()`

**Signature**: `pub async fn create_sqlite_pool(database_url: &str) -> Result<Pool<Sqlite>>`

**Purpose**: Create SQLite connection pool

---

## Usage Examples

### Example 1: MySQL Production

```rust
use auth_config::DatabaseConfig;
use auth_db::create_mysql_pool;

let config = DatabaseConfig::from_env()?;
let pool = create_mysql_pool(&config).await?;
```

### Example 2: SQLite Testing

```rust
let pool = create_sqlite_pool("sqlite::memory:").await?;
```

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database driver |
| `secrecy` | Credential protection |
| `anyhow` | Error handling |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 21  
**Security Level**: MEDIUM
