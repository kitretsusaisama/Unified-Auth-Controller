# lib.rs

## File Metadata

**File Path**: `crates/auth-cache/src/lib.rs`  
**Crate**: `auth-cache`  
**Module**: Root  
**Layer**: Infrastructure (Caching)  
**Security-Critical**: ⚠️ **MEDIUM** - Cache management

## Purpose

Provides multi-level caching with L1 (in-memory) and L2 (Redis) for performance optimization.

### Problem It Solves

- Performance optimization
- Reduced database load
- Multi-level caching
- Cache invalidation

---

## Detailed Code Breakdown

### Trait: `Cache`

**Purpose**: Cache abstraction

**Methods**:
```rust
async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Option<T>;
async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) -> anyhow::Result<()>;
async fn delete(&self, key: &str) -> anyhow::Result<()>;
```

---

### Struct: `MultiLevelCache`

**Purpose**: Two-level cache implementation

**Fields**:
- `l1`: In-memory cache (DashMap)
- `l2`: Redis cache

---

### Cache Strategy

**L1 (Memory)**:
- Fast access
- Limited size
- 60-second TTL

**L2 (Redis)**:
- Persistent
- Shared across instances
- Configurable TTL

**Flow**:
1. Check L1 → Hit: Return
2. Check L2 → Hit: Populate L1, Return
3. Miss: Return None

---

## Usage Examples

### Example 1: Cache User

```rust
let cache = MultiLevelCache::new("redis://localhost")?;

// Set
cache.set("user:123", &user, Duration::from_secs(300)).await?;

// Get
if let Some(user) = cache.get::<User>("user:123").await {
    // Cache hit
}
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 99  
**Security Level**: MEDIUM
