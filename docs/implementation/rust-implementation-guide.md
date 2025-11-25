# Rust Implementation Guide: Integer-based Node Registry

**Date**: 2025-01-25
**Status**: Implementation Specification
**Impact**: Major refactor of storage and domain layers

---

## Overview

This guide covers the Rust implementation changes required for the integer-based node registry architecture.

**Core Pattern**: "Internal Ints, External Strings"
- Internal graph operations: use `i64` node IDs
- External API: accept/return `String` unique keys
- Boundary layer (NodeRegistry): maps between the two

---

## Phase 1: Domain Model Updates

### File: `rust/crates/iqrah-core/src/domain/models.rs`

#### Update Node Struct

```rust
// BEFORE:
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub knowledge_node: Option<KnowledgeNode>,
}

// AFTER:
pub struct Node {
    pub id: i64,               // NEW: Internal integer ID
    pub ukey: String,          // NEW: External unique key
    pub node_type: NodeType,
    pub knowledge_node: Option<KnowledgeNode>,
}
```

#### Update KnowledgeNode

```rust
// BEFORE:
pub struct KnowledgeNode {
    pub base_node_id: String,
    pub axis: KnowledgeAxis,
    pub full_id: String,
}

// AFTER:
pub struct KnowledgeNode {
    pub base_node_id: i64,     // Integer ID of base node
    pub base_node_ukey: String,// String key of base node
    pub axis: KnowledgeAxis,
}
```

#### Add Integer Enum Mappings

```rust
// NodeType enum (map to INTEGER in DB)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum NodeType {
    Verse = 0,
    Chapter = 1,
    Word = 2,
    Knowledge = 3,
    WordInstance = 4,
}

impl NodeType {
    pub fn to_int(self) -> i32 {
        self as i32
    }

    pub fn from_int(val: i32) -> Result<Self> {
        match val {
            0 => Ok(NodeType::Verse),
            1 => Ok(NodeType::Chapter),
            2 => Ok(NodeType::Word),
            3 => Ok(NodeType::Knowledge),
            4 => Ok(NodeType::WordInstance),
            _ => Err(Error::InvalidNodeType(val)),
        }
    }
}

// KnowledgeAxis enum (map to INTEGER in DB)
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
pub enum KnowledgeAxis {
    Memorization = 0,
    Translation = 1,
    Tafsir = 2,
    Tajweed = 3,
    ContextualMemorization = 4,
    Meaning = 5,
}

impl KnowledgeAxis {
    pub fn to_int(self) -> i32 {
        self as i32
    }

    pub fn from_int(val: i32) -> Result<Self> {
        match val {
            0 => Ok(KnowledgeAxis::Memorization),
            1 => Ok(KnowledgeAxis::Translation),
            2 => Ok(KnowledgeAxis::Tafsir),
            3 => Ok(KnowledgeAxis::Tajweed),
            4 => Ok(KnowledgeAxis::ContextualMemorization),
            5 => Ok(KnowledgeAxis::Meaning),
            _ => Err(Error::InvalidKnowledgeAxis(val)),
        }
    }
}
```

---

## Phase 2: NodeRegistry Implementation

### File: `rust/crates/iqrah-storage/src/content/node_registry.rs` (NEW)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::{SqlitePool, query, query_scalar};
use crate::error::{Result, Error};

/// Maps between stable string keys and internal integer IDs
pub struct NodeRegistry {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    pool: SqlitePool,
}

impl NodeRegistry {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            pool,
        }
    }

    /// Get integer ID from unique key (cached)
    pub async fn get_id(&self, ukey: &str) -> Result<i64> {
        // Check cache first
        {
            let cache_read = self.cache.read().await;
            if let Some(&id) = cache_read.get(ukey) {
                return Ok(id);
            }
        }

        // Query database
        let id = query_scalar!(
            "SELECT id FROM nodes WHERE ukey = ?",
            ukey
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NodeNotFound(ukey.to_string()))?;

        // Update cache
        self.cache.write().await.insert(ukey.to_string(), id);

        Ok(id)
    }

    /// Get unique key from integer ID
    pub async fn get_ukey(&self, id: i64) -> Result<String> {
        let ukey = query_scalar!(
            "SELECT ukey FROM nodes WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| Error::NodeNotFound(format!("id={}", id)))?;

        Ok(ukey)
    }

    /// Batch lookup: string keys → integer IDs
    pub async fn get_ids(&self, ukeys: &[String]) -> Result<Vec<i64>> {
        let mut ids = Vec::with_capacity(ukeys.len());

        for ukey in ukeys {
            ids.push(self.get_id(ukey).await?);
        }

        Ok(ids)
    }

    /// Batch lookup: integer IDs → string keys
    pub async fn get_ukeys(&self, ids: &[i64]) -> Result<Vec<String>> {
        let mut ukeys = Vec::with_capacity(ids.len());

        for &id in ids {
            ukeys.push(self.get_ukey(id).await?);
        }

        Ok(ukeys)
    }

    /// Preload cache with common nodes (optimization)
    pub async fn preload_cache(&self, limit: usize) -> Result<()> {
        let nodes: Vec<(i64, String)> = query!(
            "SELECT id, ukey FROM nodes LIMIT ?",
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| (row.id, row.ukey))
        .collect();

        let mut cache_write = self.cache.write().await;
        for (id, ukey) in nodes {
            cache_write.insert(ukey, id);
        }

        Ok(())
    }

    /// Clear cache (for testing)
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}
```

### Export from Module

**File**: `rust/crates/iqrah-storage/src/content/mod.rs`

```rust
mod node_registry;
mod repository;

pub use node_registry::NodeRegistry;
pub use repository::SqliteContentRepository;
```

---

## Phase 3: Repository Layer Refactor

### File: `rust/crates/iqrah-storage/src/content/repository.rs`

#### Update Repository Struct

```rust
pub struct SqliteContentRepository {
    pool: SqlitePool,
    registry: Arc<NodeRegistry>,  // NEW: Node ID mapper
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        let registry = Arc::new(NodeRegistry::new(pool.clone()));
        Self { pool, registry }
    }

    pub async fn preload_registry(&self, limit: usize) -> Result<()> {
        self.registry.preload_cache(limit).await
    }
}
```

#### External API: String-based get_node

```rust
impl ContentRepository for SqliteContentRepository {
    /// Get node by unique key (external API)
    async fn get_node(&self, ukey: &str) -> Result<Option<Node>> {
        // 1. Lookup integer ID from string key
        let node_id = match self.registry.get_id(ukey).await {
            Ok(id) => id,
            Err(Error::NodeNotFound(_)) => return Ok(None),
            Err(e) => return Err(e),
        };

        // 2. Use fast integer path
        self.get_node_by_id(node_id).await
    }
}
```

#### Internal API: Integer-based get_node_by_id

```rust
impl SqliteContentRepository {
    /// Get node by integer ID (internal fast path)
    async fn get_node_by_id(&self, node_id: i64) -> Result<Option<Node>> {
        // Query with joins
        let row = query!(
            r#"
            SELECT
                n.id, n.ukey, n.node_type,
                v.chapter_number, v.verse_number, v.text_uthmani, v.text_simple,
                kn.base_node_id, kn.axis
            FROM nodes n
            LEFT JOIN verses v ON v.node_id = n.id
            LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
            WHERE n.id = ?
            "#,
            node_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        // Parse node type
        let node_type = NodeType::from_int(row.node_type)?;

        // Construct Node based on type
        match node_type {
            NodeType::Verse => {
                Ok(Some(Node {
                    id: row.id,
                    ukey: row.ukey,
                    node_type: NodeType::Verse,
                    knowledge_node: None,
                }))
            }

            NodeType::Knowledge => {
                let base_node_id = row.base_node_id
                    .ok_or_else(|| Error::MissingKnowledgeNodeData(node_id))?;
                let axis_int = row.axis
                    .ok_or_else(|| Error::MissingKnowledgeNodeData(node_id))?;
                let axis = KnowledgeAxis::from_int(axis_int)?;

                // Get base node ukey
                let base_node_ukey = self.registry.get_ukey(base_node_id).await?;

                Ok(Some(Node {
                    id: row.id,
                    ukey: row.ukey,
                    node_type: NodeType::Knowledge,
                    knowledge_node: Some(KnowledgeNode {
                        base_node_id,
                        base_node_ukey,
                        axis,
                    }),
                }))
            }

            // ... other node types ...
            _ => unimplemented!("Node type not yet implemented"),
        }
    }
}
```

#### Get Edges (Integer-based)

```rust
impl SqliteContentRepository {
    /// Get outgoing edges from node (returns target IDs)
    async fn get_edges(&self, source_id: i64) -> Result<Vec<Edge>> {
        let rows = query!(
            r#"
            SELECT target_id, edge_type, weight
            FROM edges
            WHERE source_id = ?
            "#,
            source_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut edges = Vec::with_capacity(rows.len());
        for row in rows {
            edges.push(Edge {
                source_id,
                target_id: row.target_id,
                edge_type: EdgeType::from_int(row.edge_type)?,
                weight: row.weight,
            });
        }

        Ok(edges)
    }

    /// Get all nodes reachable from source (BFS)
    async fn get_reachable_nodes(&self, source_id: i64, max_depth: usize) -> Result<Vec<i64>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((source_id, 0));

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= max_depth || visited.contains(&node_id) {
                continue;
            }

            visited.insert(node_id);

            // Get outgoing edges (INTEGER joins - fast!)
            let edges = self.get_edges(node_id).await?;
            for edge in edges {
                queue.push_back((edge.target_id, depth + 1));
            }
        }

        Ok(visited.into_iter().collect())
    }
}
```

---

## Phase 4: Learning Service Updates

### File: `rust/crates/iqrah-core/src/services/learning_service.rs`

#### Energy Propagation (Integer-based)

```rust
impl LearningService {
    /// Propagate energy through knowledge graph
    async fn propagate_energy(&self, source_node_id: i64, energy_amount: f64) -> Result<()> {
        // Get outgoing edges (INTEGER joins - fast!)
        let edges = query!(
            "SELECT target_id, weight FROM edges WHERE source_id = ?",
            source_node_id
        )
        .fetch_all(&self.pool)
        .await?;

        for edge in edges {
            let propagated_energy = energy_amount * edge.weight;

            // Update energy on target node (INTEGER operations)
            self.add_energy(edge.target_id, propagated_energy).await?;

            // Recursively propagate if above threshold
            if propagated_energy > 0.01 {
                Box::pin(self.propagate_energy(edge.target_id, propagated_energy)).await?;
            }
        }

        Ok(())
    }

    /// Add energy to node
    async fn add_energy(&self, node_id: i64, amount: f64) -> Result<()> {
        // Update in content.db (internal integer ID)
        query!(
            "UPDATE node_metadata SET value = value + ? WHERE node_id = ? AND key = 'energy'",
            amount,
            node_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Save user state (convert to string key at boundary)
    async fn save_user_state(&self, node_id: i64, state: MemoryState) -> Result<()> {
        // Convert integer ID to string key for user.db
        let node_ukey = self.registry.get_ukey(node_id).await?;

        // Store in user.db using STABLE STRING KEY
        query!(
            r#"
            INSERT INTO user_memory_states (user_id, node_ukey, stability, difficulty, energy)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (user_id, node_ukey) DO UPDATE SET
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                energy = excluded.energy
            "#,
            state.user_id,
            node_ukey,  // String key for stability
            state.stability,
            state.difficulty,
            state.energy
        )
        .execute(&self.user_pool)
        .await?;

        Ok(())
    }
}
```

---

## Phase 5: Error Handling

### File: `rust/crates/iqrah-core/src/error.rs`

Add new error variants:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid node type: {0}")]
    InvalidNodeType(i32),

    #[error("Invalid knowledge axis: {0}")]
    InvalidKnowledgeAxis(i32),

    #[error("Missing knowledge node data for node_id: {0}")]
    MissingKnowledgeNodeData(i64),

    // ... existing errors ...
}
```

---

## Migration Checklist

### Domain Models:
- [ ] Update `Node` struct with `id: i64` and `ukey: String`
- [ ] Update `KnowledgeNode` struct with integer IDs
- [ ] Add `to_int()` and `from_int()` methods to enums
- [ ] Add sqlx derives for integer enum mapping

### NodeRegistry:
- [ ] Create `node_registry.rs` file
- [ ] Implement `get_id()` with caching
- [ ] Implement `get_ukey()`
- [ ] Implement batch methods
- [ ] Export from `mod.rs`

### Repository:
- [ ] Add `NodeRegistry` to repository struct
- [ ] Update `get_node()` to use registry
- [ ] Add `get_node_by_id()` internal method
- [ ] Update all queries to use INTEGER IDs
- [ ] Add edge traversal methods

### Services:
- [ ] Update propagation logic to use integers
- [ ] Convert to strings only at user.db boundary
- [ ] Update all database queries

### Testing:
- [ ] Unit tests for NodeRegistry
- [ ] Unit tests for integer enum conversion
- [ ] Integration tests for repository layer
- [ ] Benchmark integer vs string performance

---

## Performance Expectations

### Before (String-based):
- Node lookup: O(n) string comparison
- Edge traversal: O(n*m) where m = string length
- Memory: 20-50 bytes per node ID

### After (Integer-based):
- Node lookup: O(1) integer comparison (with cache)
- Edge traversal: O(n) integer comparison
- Memory: 8 bytes per node ID

**Expected Improvement**: 10-100x faster graph operations

---

## References

- [Schema Design](schema-design.md) - Database schema DDL
- [Python Generator Guide](python-generator-guide.md) - Graph generation
- [Enum Mappings](../reference/enum-mappings.md) - Integer enum values
- [Validation Checklist](../reference/validation-checklist.md) - Post-implementation checks
