# sharding.rs

## File Metadata

**File Path**: `crates/auth-db/src/sharding.rs`  
**Crate**: `auth-db`  
**Module**: `sharding`  
**Layer**: Infrastructure (Database)  
**Security-Critical**: ⚠️ **MEDIUM** - Data distribution and isolation

## Purpose

Implements consistent hashing-based database sharding for horizontal scalability and multi-tenant data isolation.

### Problem It Solves

- Horizontal database scaling
- Tenant data isolation
- Load distribution
- Consistent hashing
- Shard management

---

## Detailed Code Breakdown

### Struct: `ShardConfig`

**Purpose**: Shard configuration

**Fields**:
- `shard_id`: Unique shard identifier
- `database_url`: Connection string
- `weight`: Distribution weight for consistent hashing

---

### Struct: `ShardManager`

**Purpose**: Manages database shards with consistent hashing

**Fields**:
- `pools`: Map of shard ID to connection pool
- `ring`: Consistent hashing ring (hash → shard_id)
- `validation_key`: Shard validation key

---

### Method: `add_shard()`

**Signature**: `pub async fn add_shard(&self, config: ShardConfig) -> Result<()>`

**Purpose**: Add new shard to the ring

**Process**:

1. **Create Pool**
```rust
let pool = MySqlPool::connect_lazy(&config.database_url)?;
pools.insert(config.shard_id, pool);
```

2. **Add Virtual Nodes**
```rust
let virtual_nodes = 100 * config.weight;
for i in 0..virtual_nodes {
    let key = format!("{}:{}", config.shard_id, i);
    let hash = self.hash_key(&key);
    ring.push((hash, config.shard_id));
}
ring.sort_by(|a, b| a.0.cmp(&b.0));
```

---

### Method: `get_pool()`

**Signature**: `pub async fn get_pool(&self, tenant_id: Uuid) -> Option<MySqlPool>`

**Purpose**: Get connection pool for tenant

**Process**:
1. Determine shard ID using consistent hashing
2. Return corresponding pool

---

### Method: `determine_shard_id()`

**Signature**: `pub async fn determine_shard_id(&self, key: &str) -> Option<u32>`

**Purpose**: Determine shard using consistent hashing

**Algorithm**:
```rust
let hash = self.hash_key(key);
// Binary search in sorted ring
match ring.binary_search_by(|(h, _)| h.cmp(&hash)) {
    Ok(idx) => Some(ring[idx].1),
    Err(idx) => {
        if idx == ring.len() {
            Some(ring[0].1) // Wrap around
        } else {
            Some(ring[idx].1)
        }
    }
}
```

---

## Consistent Hashing

### Why Consistent Hashing?

**Benefits**:
- Minimal data movement when adding/removing shards
- Even distribution
- Predictable tenant placement

### Virtual Nodes

**Purpose**: Improve distribution uniformity

```rust
// Each shard gets 100 * weight virtual nodes
let virtual_nodes = 100 * config.weight;
```

---

## Usage Examples

### Example 1: Setup Shards

```rust
let shard_manager = ShardManager::new();

// Add shard 1
shard_manager.add_shard(ShardConfig {
    shard_id: 1,
    database_url: "mysql://shard1.example.com/db".to_string(),
    weight: 1,
}).await?;

// Add shard 2 with higher weight
shard_manager.add_shard(ShardConfig {
    shard_id: 2,
    database_url: "mysql://shard2.example.com/db".to_string(),
    weight: 2, // Gets 2x virtual nodes
}).await?;
```

### Example 2: Get Tenant Pool

```rust
let tenant_id = Uuid::new_v4();
let pool = shard_manager.get_pool(tenant_id).await
    .ok_or(anyhow!("No shard available"))?;

// Use pool for tenant queries
let users = sqlx::query_as!(User, "SELECT * FROM users WHERE tenant_id = ?", tenant_id)
    .fetch_all(&pool)
    .await?;
```

---

## Security Considerations

### 1. Tenant Isolation

**Guarantee**: Each tenant always routes to same shard

```rust
// Tenant A always goes to shard 1
let shard_a = shard_manager.determine_shard_id(&tenant_a_id.to_string()).await;

// Tenant B always goes to shard 2
let shard_b = shard_manager.determine_shard_id(&tenant_b_id.to_string()).await;
```

### 2. Data Leakage Prevention

**Requirement**: Never query across shards

---

## Dependencies

### External Crates

| Crate | Purpose |
|-------|---------|
| `sqlx` | Database pools |
| `uuid` | Tenant IDs |
| `tokio` | Async runtime |

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 89  
**Security Level**: MEDIUM
