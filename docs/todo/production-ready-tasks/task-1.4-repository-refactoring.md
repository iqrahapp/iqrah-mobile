# Task 1.4: Refactor Repository for Integer-Based Architecture

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 2 days
- **Dependencies:** Schema update to integer IDs must be complete.
- **Agent Type:** Implementation + Refactoring
- **Parallelizable:** No (Core architectural change)

## Goal

**COMPLETE REWRITE**: Refactor `SqliteContentRepository` to use the `NodeRegistry` and integer-based IDs for all internal graph operations. This task implements the "Internal Ints, External Strings" principle at the data access layer, replacing all string-based queries with high-performance integer-based lookups.

## Context

The architecture has shifted from virtual nodes with string IDs to physical nodes with integer IDs. The repository layer is where this transition is most critical. The previous implementation performed queries using slow and brittle string comparisons. This task refactors the entire repository to leverage the performance and referential integrity of the new integer-based schema.

**Old Approach (DEPRECATED):**
- Public methods accepted string IDs.
- Internal queries used those same string IDs (`WHERE node_id = 'VERSE:1:1'`).
- Knowledge nodes were wrongly assumed to be stored in `node_metadata` with a string key.

**New Approach (MANDATORY):**
- Public methods accept stable string **unique keys (ukeys)**.
- A `NodeRegistry` is used to resolve the ukey to a volatile internal **integer ID**.
- All internal methods and database queries use the fast integer ID (`WHERE n.id = ?`).

## Target State

### Node Lookup Strategy

The repository will implement a two-level lookup strategy: a public, string-based API for external callers and a private, integer-based API for internal performance.

#### External API: String-based `get_node`

This method is the entry point for all external services. It takes a stable `ukey` and uses the `NodeRegistry` to find the corresponding integer ID.

```rust
async fn get_node(&self, ukey: &str) -> Result<Option<Node>> {
    // 1. Lookup integer ID from string key via the registry
    let node_id_opt = self.registry.get_id(ukey).await?;

    if let Some(node_id) = node_id_opt {
        // 2. Use the fast, internal integer-based path
        self.get_node_by_id(node_id).await
    } else {
        Ok(None)
    }
}
```

#### Internal API: Integer-based `get_node_by_id`

This is the core data retrieval method. It queries the database using the high-performance integer primary key.

```rust
async fn get_node_by_id(&self, node_id: i64) -> Result<Option<Node>> {
    let row = sqlx::query!(
        r#"
        SELECT n.id, n.ukey, n.node_type, kn.base_node_id, kn.axis
        FROM nodes n
        LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
        WHERE n.id = ?
        "#,
        node_id
    ).fetch_optional(&self.pool).await?;

    // Logic to construct a Node from the query row...
    // This will involve mapping integer enums back to Rust enums.
}
```

### NodeRegistry Integration

The repository must be initialized with a `NodeRegistry` instance to perform the ukey-to-ID mapping.

```rust
pub struct SqliteContentRepository {
    pool: SqlitePool,
    registry: Arc<NodeRegistry>,
}

pub struct NodeRegistry {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    pool: SqlitePool,
}
```

## Implementation Steps

### Step 1: Update `SqliteContentRepository` Struct (30 min)

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

- Add the `registry: Arc<NodeRegistry>` field to the struct.
- Update the constructor (`new()`) to accept and store the `NodeRegistry`.

### Step 2: Implement the New Lookup Strategy (3-4 hours)

- **Rewrite `get_node()`** to match the target state example. It should now delegate to the `NodeRegistry` and `get_node_by_id()`.
- **Create `get_node_by_id()`** to perform the integer-based SQL query. Ensure it correctly joins `nodes` and `knowledge_nodes`.

### Step 3: Refactor All Other Repository Methods (4-5 hours)

Scour the entire repository for any method that queries or interacts with nodes or edges. Every single one must be converted to use integer IDs.

**Example: Refactoring `get_edges_from`**

```rust
// OLD (string-based)
// async fn get_edges_from(&self, source_node_id: &str) -> Result<Vec<Edge>>

// NEW (integer-based)
async fn get_edges_from(&self, source_node_id: i64) -> Result<Vec<Edge>> {
    let edges = sqlx::query_as!(
        Edge,
        "SELECT source_id, target_id, edge_type, weight FROM edges WHERE source_id = ?",
        source_node_id
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(edges)
}
```

- Any public method that previously took a string `node_id` must now be updated to either:
    1.  Keep the `&str` signature but immediately use the registry to convert it to an integer.
    2.  Be changed to accept an `i64` if it's only called internally by other repository methods.

### Step 4: Update Tests (2-3 hours)

- Tests for public methods should still pass string `ukeys`.
- Mocks or test setup for the `NodeRegistry` will be required.
- Tests for internal methods should be updated to work with integer IDs.
- Add tests to verify that a non-existent `ukey` returns `Ok(None)`.

## Verification Plan

### Correctness Checklist

- [ ] `SqliteContentRepository` holds a `NodeRegistry`.
- [ ] `get_node(ukey: &str)` exists and uses the registry.
- [ ] `get_node_by_id(id: i64)` exists and uses an integer-based query.
- [ ] **ALL** other data access methods (`get_edges`, `get_metadata`, etc.) have been refactored to use `i64` IDs in their SQL queries.
- [ ] All references to virtual nodes or string-based node queries are **REMOVED**.
- [ ] All unit and integration tests pass.

### Verification Queries

Run these checks manually against the code:

```bash
# Search for string-based queries that need to be removed.
# This should yield 0 results after the refactor.
cd rust/crates/iqrah-storage/
rg "WHERE node_id = \?" src/
rg "WHERE source_id = \?" src/
rg "WHERE base_node_id = \?" src/
```

## Scope Limits & Safeguards

### ✅ MUST DO

- Remove **ALL** "virtual node" references and logic.
- Update **ALL** code examples to use integer IDs for database operations.
- Document and implement the `NodeRegistry` usage for lookups.
- Update **ALL** SQL queries to use `INTEGER` joins and `WHERE` clauses (e.g., `WHERE n.id = ?`).

### ❌ DO NOT

- Change the public API signatures where they are consumed by external services. They should still accept string `ukeys`.
- Change the database schema. This task adapts to the new schema.
- Implement caching beyond what `NodeRegistry` provides.

## Success Criteria

- [ ] All references to virtual nodes are purged from the repository.
- [ ] All SQL queries use integer IDs for node and edge operations.
- [ ] The `get_node` method correctly implements the ukey-to-ID lookup flow.
- [ ] The repository code is fully aligned with the "Internal Ints, External Strings" principle.
- [ ] All CI checks (build, clippy, test, fmt) pass.
