# Knowledge Node Storage Architecture: Design Decision

**Date**: 2025-01-25
**Status**: APPROVED - Ready for Implementation
**Decision By**: Project Architect
**Impact**: Critical - Affects entire graph architecture

---

## Executive Summary

**Decision**: Implement Physical Nodes with Integer-based Graph (Enhanced Option 4)

**Core Principle**: "Internal Ints, External Strings"
- Use INTEGER primary keys for graph operations (O(1) performance)
- Use STRING unique keys for stability (user data survives content updates)
- Strict referential integrity via Node Registry

---

## Background

During verification of production-ready task documentation, a fundamental architectural inconsistency was discovered:
- **Documentation** described knowledge nodes as physical database rows
- **Implementation** used virtual wrappers constructed on-demand

The virtual node implementation was clever MVP technical debt that needs to be replaced with production-grade architecture.

---

## Why Change from Virtual Nodes?

### Problems with Virtual Nodes:
1. **No Referential Integrity**: Can create orphan references (scores for non-existent nodes)
2. **Performance**: Requires parsing + recursive queries for every node access
3. **Validation**: Cannot list or count nodes directly - must infer from metadata
4. **Debugging**: Difficult to validate graph correctness without node registry

### Why Now?
- Sprint 7 is the correct time to introduce breaking schema changes
- Need production-grade architecture before scaling
- Python generator rebuild is already planned (monthly cycle)
- User database already uses stable string keys (no migration needed)

---

## Architectural Ruling: "Internal Ints, External Strings"

### The Two-Database Architecture:

#### content.db (Immutable Graph):
```sql
-- Node Registry: Source of Truth
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,        -- Internal RowID for O(1) lookups
    ukey TEXT NOT NULL UNIQUE,     -- Stable string key (e.g., "VERSE:1:1:memorization")
    node_type INTEGER NOT NULL     -- Enum: 0=verse, 1=chapter, 2=word, 3=knowledge
) STRICT;

-- All relationships use INTEGER for performance
CREATE TABLE edges (
    source_id INTEGER NOT NULL,    -- FK to nodes.id
    target_id INTEGER NOT NULL,    -- FK to nodes.id
    edge_type INTEGER NOT NULL,
    weight REAL NOT NULL,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

#### user.db (Mutable User State):
```sql
-- User state uses STABLE STRING KEYS
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_ukey TEXT NOT NULL,       -- String key (stable across content updates)
    stability REAL NOT NULL,
    difficulty REAL NOT NULL,
    energy REAL NOT NULL,
    PRIMARY KEY (user_id, node_ukey)
) STRICT, WITHOUT ROWID;
```

### Why This Design?

**Performance**: Graph algorithms operate on integers (adjacency lists, BFS/DFS, propagation)
- Integer hash lookups: O(1)
- String hash lookups: O(n) where n = string length
- Edge traversal 10-100x faster with integers

**Stability**: User data survives content.db replacements
- content.db can be regenerated monthly (integer IDs change)
- user.db persists across updates (string keys stable)
- No user data migration needed

**Referential Integrity**: Database enforces correctness
- Foreign key constraints prevent orphan edges
- Cannot insert edge without registered nodes
- Validation at schema level, not application level

---

## Key Design Elements

### 1. Node Registry Pattern

The `nodes` table is the **single source of truth** for all graph entities:
- Content nodes: `VERSE:1:1`, `CHAPTER:1`, `WORD_INSTANCE:1:1:3`
- Knowledge nodes: `VERSE:1:1:memorization`, `VERSE:1:1:translation`
- Future node types can be added without schema changes

### 2. Boundary Layer (NodeRegistry)

```rust
pub struct NodeRegistry {
    cache: RwLock<HashMap<String, i64>>,
    pool: SqlitePool,
}

// External API: string-based (stable)
async fn get_node(&self, ukey: &str) -> Result<Node>

// Internal API: integer-based (fast)
async fn get_node_by_id(&self, id: i64) -> Result<Node>
```

### 3. Knowledge Node Linking

```sql
CREATE TABLE knowledge_nodes (
    node_id INTEGER PRIMARY KEY,   -- FK to nodes.id
    base_node_id INTEGER NOT NULL, -- FK to nodes.id (underlying content)
    axis INTEGER NOT NULL,         -- Enum: 0=memorization, 1=translation, etc.
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    FOREIGN KEY (base_node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

Knowledge nodes are now **physical entities** with:
- Entry in `nodes` table (registered)
- Link to base node (referential integrity)
- Explicit axis definition (queryable)

---

## Migration Strategy

### Content DB: Complete Rebuild
- Python generator creates new schema from scratch
- Two-phase generation: register nodes, then create edges
- Timeline: Next monthly content update

### User DB: NO CHANGES NEEDED
- Already uses string keys (`node_ukey`)
- Stable across content.db updates
- Zero migration required

---

## Benefits Summary

**Performance**:
- 10-100x faster graph traversal (integer adjacency lists)
- O(1) node lookups with caching
- Efficient propagation algorithms

**Correctness**:
- Database-level referential integrity
- No orphan edges or metadata
- Validation at schema level

**Maintainability**:
- Clear separation: content vs user data
- Easy debugging (can list/count nodes)
- Future-proof (extensible node types)

**User Data Stability**:
- User progress survives content updates
- No migration on monthly content refreshes
- String keys immune to integer ID changes

---

## Implementation Timeline

**Total**: 1-2 days with 7-8 agents in parallel

- **Wave 1**: Schema design (blocking)
- **Wave 2**: Rust + Python implementation (parallel)
- **Wave 3**: Integration testing
- **Wave 4**: Documentation updates (parallel)
- **Wave 5**: Final validation

See [Agent Tasks](/docs/agent-tasks/) for detailed breakdown.

---

## Success Criteria

- [ ] All pre-commit CI checks pass
- [ ] Foreign key constraints enforced
- [ ] Performance benchmarks improved
- [ ] User data stable across content.db replacement
- [ ] All 15 production-ready tasks updated
- [ ] Zero regressions in existing functionality

---

## References

- [Design Drift Analysis](01-design-drift-analysis.md) - Original discovery
- [Schema Design](../implementation/schema-design.md) - Detailed DDL
- [Rust Implementation Guide](../implementation/rust-implementation-guide.md)
- [Python Generator Guide](../implementation/python-generator-guide.md)
