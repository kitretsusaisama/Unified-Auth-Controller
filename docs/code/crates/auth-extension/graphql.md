# graphql.rs

## File Metadata

**File Path**: `crates/auth-extension/src/graphql.rs`  
**Crate**: `auth-extension`  
**Module**: `graphql`  
**Layer**: Extension (API)  
**Security-Critical**: ⚠️ **MEDIUM** - GraphQL API

## Purpose

Provides GraphQL API extension for querying authentication data using async-graphql.

### Problem It Solves

- GraphQL API access
- Flexible querying
- Type-safe API
- Schema introspection

---

## Detailed Code Breakdown

### Struct: `QueryRoot`

**Purpose**: GraphQL query root

**Queries**:
- `user(id: Uuid)`: Fetch user by ID
- `version()`: API version

---

### Type: `ExtensionSchema`

**Purpose**: GraphQL schema type

---

### Function: `create_schema()`

**Purpose**: Create GraphQL schema

**Example**:
```rust
let schema = create_schema();
let query = "{ user(id: \"...\") { username email } }";
let result = schema.execute(query).await;
```

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-13  
**Lines of Code**: 35  
**Security Level**: MEDIUM
