# Knowledge Node Storage: Design Drift Analysis

**Date**: 2025-01-25
**Status**: Historical Record
**Purpose**: Document the discovery of architectural inconsistency

---

## Discovery Summary

During verification of production-ready task documentation, a fundamental architectural inconsistency was discovered between what the documentation describes and what the Rust implementation actually does.

**The Drift**:
- **Documentation claims**: Knowledge nodes are stored as physical database rows in `node_metadata`
- **Implementation reality**: Knowledge nodes are virtual wrappers constructed on-demand

---

## What Are Knowledge Nodes?

### Concept
Knowledge nodes represent **learning dimensions** applied to content nodes:

**Example**: For verse "VERSE:1:1", we create:
- `VERSE:1:1:memorization` - Practice reciting from memory
- `VERSE:1:1:translation` - Practice understanding the meaning
- `VERSE:1:1:tafsir` - Practice deeper contextual understanding
- `VERSE:1:1:tajweed` - Practice pronunciation rules

### The 6 Knowledge Axes
- **Verse-level (4)**: memorization, translation, tafsir, tajweed
- **Word-level (2)**: contextual_memorization, meaning

---

## Current Implementation (Before Redesign)

### Database Schema
```sql
-- Domain-specific content tables
CREATE TABLE verses (
    verse_key TEXT PRIMARY KEY,  -- e.g., "1:1" (unprefixed)
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    text_uthmani TEXT NOT NULL,
    text_simple TEXT NOT NULL
) STRICT;

-- Key-value store for graph metadata
CREATE TABLE node_metadata (
    node_id TEXT NOT NULL,        -- Can be ANY node ID
    key TEXT NOT NULL,             -- e.g., 'foundational_score'
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key)
) STRICT, WITHOUT ROWID;

-- Graph edges
CREATE TABLE edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL,
    -- ... distribution fields ...
    PRIMARY KEY (source_id, target_id)
) STRICT, WITHOUT ROWID;
```

### Virtual Node Implementation

**File**: `rust/crates/iqrah-storage/src/content/repository.rs`

```rust
async fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
    let node_type = nid::node_type(node_id)?;

    match node_type {
        NodeType::Verse => {
            // Query verses table directly
            let (chapter, verse) = nid::parse_verse(node_id)?;
            let verse_key = format!("{}:{}", chapter, verse);
            query_as("SELECT * FROM verses WHERE verse_key = ?")
                .bind(verse_key)
                .fetch_optional(&self.pool)
                .await?
        }

        NodeType::Knowledge => {
            let (base_id, axis) = nid::parse_knowledge(node_id)?;

            // CRITICAL: Recursively get base node, wrap with axis
            if let Some(mut node) = Box::pin(self.get_node(&base_id)).await? {
                node.id = node_id.to_string();
                node.node_type = NodeType::Knowledge;
                node.knowledge_node = Some(KnowledgeNode {
                    base_node_id: base_id,
                    axis,
                    full_id: node_id.to_string(),
                });
                Ok(Some(node))
            } else {
                Ok(None)
            }
        }
    }
}
```

**How it works**:
1. Parse knowledge node ID → extract base node + axis
2. Recursively fetch base node (verse)
3. Wrap base node with axis metadata
4. Return as virtual knowledge node

**Storage**:
- Content: `verses` table
- Scores: `node_metadata` stores scores for knowledge node IDs
- Edges: `edges` table references knowledge node IDs
- No dedicated knowledge node table

---

## What Documentation Claimed

### Task 1.4 (Repository Refactoring) - Lines 400-405:
```
Knowledge nodes like "VERSE:1:1:memorization" ARE stored in the database
in the node_metadata table with their full ID as the primary key.
They are NOT virtual nodes - they are real database entities with:
- node_id: e.g., "VERSE:1:1:memorization"
- Scores: foundational_score, influence_score, etc.
- Graph edges connecting them

Query them directly: SELECT * FROM node_metadata WHERE node_id = 'VERSE:1:1:memorization'
```

### Task 2.1 (Knowledge Graph Generation) - Lines 237-241:
```sql
-- Knowledge nodes in node_metadata
INSERT INTO node_metadata (node_id, key, value) VALUES
    ('VERSE:1:1:memorization', 'foundational_score', 0.0123),
    ('VERSE:1:1:translation', 'foundational_score', 0.0098);
```

**Documentation implied**:
- Knowledge nodes are physical database entities
- `node_metadata` stores node objects, not just scores
- Can query knowledge nodes as rows

---

## Detailed Comparison

### Approach A: Virtual Nodes (Current Implementation)

#### Data Flow
```
User requests: "VERSE:1:1:memorization"
                    ↓
         Parse node ID format
                    ↓
    base_id="VERSE:1:1", axis=Memorization
                    ↓
    Fetch verse "1:1" from verses table
                    ↓
    Return verse wrapped as knowledge node
```

#### Characteristics

**Pros**:
- No data duplication (verse stored once)
- Simple schema (no new tables)
- Knowledge axis is just ID parsing logic
- Content changes automatically reflected
- Elegant separation: storage ≠ learning dimensions

**Cons**:
- Knowledge nodes don't "exist" until requested
- Can't do `SELECT * FROM knowledge_nodes`
- Slightly slower (parsing + recursion)
- No database-level validation
- Can create orphan references

---

### Approach B: Physical Storage (Documentation Version)

#### How It Would Work
```
During graph generation:
    For each verse "VERSE:1:1":
        For each axis in [memorization, translation, tafsir, tajweed]:
            INSERT INTO knowledge_nodes VALUES (
                'VERSE:1:1:{axis}',
                'VERSE:1:1',  -- base_node_id
                '{axis}'       -- axis type
            )
```

#### Required Schema
```sql
CREATE TABLE knowledge_nodes (
    node_id TEXT PRIMARY KEY,          -- e.g., "VERSE:1:1:memorization"
    base_node_id TEXT NOT NULL,        -- e.g., "VERSE:1:1"
    axis TEXT NOT NULL,                 -- e.g., "memorization"
    FOREIGN KEY (base_node_id) REFERENCES verses(verse_key)
) STRICT;
```

#### Characteristics

**Pros**:
- Knowledge nodes are "real" database entities
- Can query: `SELECT * FROM knowledge_nodes`
- Database enforces referential integrity
- No parsing/recursion overhead
- More intuitive for SQL-centric developers

**Cons**:
- Data duplication (493 verses × 4 axes = ~2000 rows)
- Must create/maintain knowledge node rows
- Content and knowledge dimensions tightly coupled
- Requires explicit INSERT during generation
- Schema migration required

---

## What is `node_metadata` For?

### Two Interpretations:

**Interpretation 1: KV Store for Metadata (Current)**
- Stores graph scores for any node ID
- Node IDs are just strings
- Analogous to Redis (key-value store)

```sql
-- Store score for any node ID (doesn't need to "exist")
INSERT INTO node_metadata VALUES ('VERSE:1:1:memorization', 'foundational_score', 0.123);
```

**Interpretation 2: Node Object Storage (Documentation)**
- Stores node entities themselves
- Node must "exist" in this table to be valid
- Analogous to MongoDB (document store)

```sql
-- Create a knowledge node entity
INSERT INTO node_metadata (node_id, ...) VALUES ('VERSE:1:1:memorization', ...);
```

### User's Concern
`node_metadata` was designed as an **MVP artifact** for attaching additional data to nodes, not for storing nodes themselves.

---

## Why Virtual Nodes Were Problematic

1. **No Referential Integrity**: Could create scores/edges for non-existent knowledge nodes
2. **Validation Difficulty**: Cannot list or count knowledge nodes directly
3. **Performance**: Parsing + recursive queries for every access
4. **Debugging**: Difficult to validate graph correctness
5. **Production Scalability**: Not suitable for large-scale operations

---

## Resolution Options Considered

### Option 1: Update Documentation to Match Implementation
- Fix 5+ task documents to describe virtual nodes
- Timeline: 1-2 hours
- Risk: Low

### Option 2: Update Implementation to Match Documentation
- Implement physical knowledge node storage
- Timeline: 2-3 days
- Risk: Medium-High

### Option 3: Hybrid Approach
- Keep virtual nodes, rename `node_metadata` for clarity
- Timeline: 1 day
- Risk: Low-Medium

### Option 4: Extended Physical Storage (CHOSEN)
- Create comprehensive node registry with integer optimization
- Timeline: 1-2 weeks (with parallel agents: 1-2 days)
- Risk: High but necessary for production

---

## Decision Rationale

**Why Option 4 was chosen**:
1. Virtual nodes were MVP technical debt
2. Need database referential integrity for production
3. Integer-based graph provides significant performance gains
4. User data stability requires separation of concerns
5. Sprint 7 is the right time for breaking changes

See [00-knowledge-node-redesign-decision.md](00-knowledge-node-redesign-decision.md) for the approved architecture.

---

## Affected Documentation Files

**Must update**:
- task-1.4-repository-refactoring.md (major errors)
- task-2.1-generate-full-knowledge-graph.md (incorrect SQL)
- task-2.2-verify-knowledge-axis-end-to-end.md (wrong assumptions)
- task-2.4-cross-axis-propagation-verification.md (wrong assumptions)

**Should review**:
- task-1.1-architecture-documentation.md (add clarification)
- task-3.3-graph-update-mechanism.md (verify assumptions)

---

## Lessons Learned

1. **Documentation must match implementation**: Drift creates confusion for future developers/agents
2. **Virtual resources have limits**: Clever shortcuts become technical debt
3. **Validate assumptions early**: Review production-ready tasks against actual code
4. **Referential integrity matters**: Database constraints prevent entire classes of bugs
5. **Performance optimization timing**: Integer IDs best introduced during initial architecture, not later

---

## References

- Current implementation: `rust/crates/iqrah-storage/src/content/repository.rs`
- Domain models: `rust/crates/iqrah-core/src/domain/models.rs`
- ID parsing: `rust/crates/iqrah-core/src/domain/node_id.rs`
