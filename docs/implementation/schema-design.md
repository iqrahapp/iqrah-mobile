# Knowledge Graph Schema Design: Integer-based Node Registry

**Date**: 2025-01-25
**Status**: Production Specification
**Database**: content.db (immutable graph structure)

---

## Schema Overview

### Core Principle: Integer IDs for Performance

All graph operations use INTEGER primary keys for O(1) lookups and efficient joins. String unique keys provide stability for external references (user.db).

---

## Complete DDL

### Node Registry (Source of Truth)

```sql
-- Node Registry: Central authority for all graph entities
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,        -- Internal RowID (auto-assigned by SQLite)
    ukey TEXT NOT NULL UNIQUE,     -- Stable string key (e.g., "VERSE:1:1:memorization")
    node_type INTEGER NOT NULL,    -- Enum: 0=verse, 1=chapter, 2=word, 3=knowledge, 4=word_instance
    CHECK (node_type >= 0 AND node_type <= 4)
) STRICT;

-- Index for fast string lookups
CREATE UNIQUE INDEX idx_nodes_ukey ON nodes(ukey);

-- Index for node type queries
CREATE INDEX idx_nodes_type ON nodes(node_type);
```

**Design Notes**:
- `id`: SQLite RowID, auto-increments, used for all FKs
- `ukey`: Human-readable stable identifier, used in user.db
- `node_type`: Enum value (see [enum-mappings.md](../reference/enum-mappings.md))

---

### Knowledge Node Definitions

```sql
-- Links knowledge nodes to their base content nodes
CREATE TABLE knowledge_nodes (
    node_id INTEGER PRIMARY KEY,   -- FK to nodes.id (must be NodeType::Knowledge)
    base_node_id INTEGER NOT NULL, -- FK to nodes.id (content node)
    axis INTEGER NOT NULL,         -- Enum: 0=memorization, 1=translation, etc.
    CHECK (axis >= 0 AND axis <= 5),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (base_node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for axis queries
CREATE INDEX idx_knowledge_nodes_axis ON knowledge_nodes(axis);

-- Index for base node lookups
CREATE INDEX idx_knowledge_nodes_base ON knowledge_nodes(base_node_id);
```

**Design Notes**:
- Every knowledge node must have entry in `nodes` table first
- `axis` is the learning dimension (see [enum-mappings.md](../reference/enum-mappings.md))
- Cascading delete: removing base node removes knowledge nodes

---

### Graph Edges

```sql
-- Weighted directed edges between nodes
CREATE TABLE edges (
    source_id INTEGER NOT NULL,    -- FK to nodes.id
    target_id INTEGER NOT NULL,    -- FK to nodes.id
    edge_type INTEGER NOT NULL,    -- Enum: 0=dependency, 1=knowledge
    weight REAL NOT NULL DEFAULT 0.5,
    CHECK (edge_type >= 0 AND edge_type <= 1),
    CHECK (weight >= 0.0 AND weight <= 1.0),
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for forward traversal
CREATE INDEX idx_edges_source ON edges(source_id);

-- Index for backward traversal
CREATE INDEX idx_edges_target ON edges(target_id);
```

**Design Notes**:
- Composite PK prevents duplicate edges
- WITHOUT ROWID for memory efficiency
- Indexes support both BFS/DFS directions

---

### Content Tables (Linked via Integer IDs)

#### Verses

```sql
-- Quranic verse content
CREATE TABLE verses (
    node_id INTEGER PRIMARY KEY,   -- FK to nodes.id
    chapter_number INTEGER NOT NULL,
    verse_number INTEGER NOT NULL,
    text_uthmani TEXT NOT NULL,
    text_simple TEXT NOT NULL,
    juz_number INTEGER NOT NULL,
    hizb_number INTEGER NOT NULL,
    rub_number INTEGER NOT NULL,
    CHECK (chapter_number >= 1 AND chapter_number <= 114),
    CHECK (verse_number >= 1),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for chapter/verse lookups
CREATE UNIQUE INDEX idx_verses_location ON verses(chapter_number, verse_number);
```

#### Words

```sql
-- Word instances in verses
CREATE TABLE words (
    node_id INTEGER PRIMARY KEY,   -- FK to nodes.id
    verse_key TEXT NOT NULL,       -- e.g., "1:1" (for join to verses)
    position INTEGER NOT NULL,     -- Word position in verse
    text_uthmani TEXT NOT NULL,
    text_simple TEXT NOT NULL,
    translation TEXT,
    transliteration TEXT,
    CHECK (position >= 1),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for verse lookups
CREATE INDEX idx_words_verse ON words(verse_key);
```

---

### Node Metadata (Scores)

```sql
-- Key-value store for graph algorithm scores
CREATE TABLE node_metadata (
    node_id INTEGER NOT NULL,      -- FK to nodes.id
    key TEXT NOT NULL,             -- e.g., 'foundational_score', 'influence_score'
    value REAL NOT NULL,
    PRIMARY KEY (node_id, key),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for key-based queries
CREATE INDEX idx_node_metadata_key ON node_metadata(key);
```

**Design Notes**:
- Flexible KV store for algorithm metadata
- Common keys: `foundational_score`, `influence_score`, `difficulty_score`

---

### Node Goals (Learning Objectives)

```sql
-- Maps learning goals to nodes
CREATE TABLE node_goals (
    goal_id TEXT NOT NULL,         -- e.g., "surah_al_fatiha", "daily_review"
    node_id INTEGER NOT NULL,      -- FK to nodes.id
    priority INTEGER DEFAULT 0,
    CHECK (priority >= 0),
    PRIMARY KEY (goal_id, node_id),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT, WITHOUT ROWID;

-- Index for goal lookups
CREATE INDEX idx_node_goals_goal ON node_goals(goal_id);
```

---

## Example Data

### Node Registration

```sql
-- Register content nodes
INSERT INTO nodes (id, ukey, node_type) VALUES
    (1, 'CHAPTER:1', 1),                    -- Chapter: Al-Fatiha
    (2, 'VERSE:1:1', 0),                    -- Verse 1:1
    (3, 'VERSE:1:2', 0),                    -- Verse 1:2
    (4, 'WORD_INSTANCE:1:1:1', 4),          -- First word of 1:1
    (5, 'WORD_INSTANCE:1:1:2', 4);          -- Second word of 1:1

-- Register knowledge nodes
INSERT INTO nodes (id, ukey, node_type) VALUES
    (101, 'VERSE:1:1:memorization', 3),
    (102, 'VERSE:1:1:translation', 3),
    (103, 'VERSE:1:1:tafsir', 3),
    (104, 'VERSE:1:1:tajweed', 3);

-- Define knowledge node links
INSERT INTO knowledge_nodes (node_id, base_node_id, axis) VALUES
    (101, 2, 0),  -- memorization of VERSE:1:1
    (102, 2, 1),  -- translation of VERSE:1:1
    (103, 2, 2),  -- tafsir of VERSE:1:1
    (104, 2, 3);  -- tajweed of VERSE:1:1
```

### Content Storage

```sql
-- Store verse content
INSERT INTO verses (node_id, chapter_number, verse_number, text_uthmani, text_simple, juz_number, hizb_number, rub_number)
VALUES (2, 1, 1, 'بِسْمِ ٱللَّهِ ٱلرَّحْمَـٰنِ ٱلرَّحِيمِ', 'بسم الله الرحمن الرحيم', 1, 1, 1);

-- Store word
INSERT INTO words (node_id, verse_key, position, text_uthmani, text_simple)
VALUES (4, '1:1', 1, 'بِسْمِ', 'بسم');
```

### Graph Structure

```sql
-- Edges between knowledge nodes (dependency graph)
INSERT INTO edges (source_id, target_id, edge_type, weight) VALUES
    (101, 102, 0, 0.8),  -- memorization → translation
    (102, 103, 0, 0.7),  -- translation → tafsir
    (101, 104, 0, 0.6);  -- memorization → tajweed

-- Scores
INSERT INTO node_metadata (node_id, key, value) VALUES
    (101, 'foundational_score', 0.0123),
    (101, 'influence_score', 0.0456),
    (102, 'foundational_score', 0.0098);

-- Goals
INSERT INTO node_goals (goal_id, node_id, priority) VALUES
    ('surah_al_fatiha', 101, 10),
    ('surah_al_fatiha', 102, 10);
```

---

## Migration SQL Template

**File**: `rust/migrations_content/YYYYMMDD_node_registry.sql`

```sql
-- Migration: Integer-based Node Registry
-- Date: YYYY-MM-DD
-- Breaking change: Requires full content.db rebuild

-- Drop old tables (if exists)
DROP TABLE IF EXISTS node_goals;
DROP TABLE IF EXISTS node_metadata;
DROP TABLE IF EXISTS edges;
DROP TABLE IF EXISTS words;
DROP TABLE IF EXISTS verses;
DROP TABLE IF EXISTS chapters;
DROP TABLE IF EXISTS knowledge_nodes;
DROP TABLE IF EXISTS nodes;

-- Create new schema (as defined above)
CREATE TABLE nodes (...);
CREATE TABLE knowledge_nodes (...);
CREATE TABLE edges (...);
CREATE TABLE verses (...);
CREATE TABLE words (...);
CREATE TABLE node_metadata (...);
CREATE TABLE node_goals (...);

-- Create indexes
CREATE UNIQUE INDEX idx_nodes_ukey ON nodes(ukey);
CREATE INDEX idx_nodes_type ON nodes(node_type);
-- ... (all other indexes) ...

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;
```

---

## Validation Queries

### Check Referential Integrity

```sql
-- Find orphan edges (should return 0)
SELECT COUNT(*) FROM edges e
LEFT JOIN nodes n1 ON e.source_id = n1.id
LEFT JOIN nodes n2 ON e.target_id = n2.id
WHERE n1.id IS NULL OR n2.id IS NULL;

-- Find orphan knowledge nodes (should return 0)
SELECT COUNT(*) FROM knowledge_nodes kn
LEFT JOIN nodes n ON kn.node_id = n.id
WHERE n.id IS NULL;

-- Find orphan metadata (should return 0)
SELECT COUNT(*) FROM node_metadata nm
LEFT JOIN nodes n ON nm.node_id = n.id
WHERE n.id IS NULL;
```

### Count Nodes by Type

```sql
SELECT node_type, COUNT(*) as count
FROM nodes
GROUP BY node_type;

-- Expected output (chapters 1-3):
-- 0 (Verse): ~143
-- 1 (Chapter): 3
-- 2 (Word): ~500+
-- 3 (Knowledge): ~572 (143 verses × 4 axes)
-- 4 (WordInstance): ~500+
```

### Query Knowledge Nodes

```sql
-- All memorization nodes
SELECT n.ukey, kn.axis
FROM nodes n
JOIN knowledge_nodes kn ON n.id = kn.node_id
WHERE kn.axis = 0;  -- Memorization

-- Knowledge nodes for specific verse
SELECT n.ukey, kn.axis
FROM nodes n
JOIN knowledge_nodes kn ON n.id = kn.node_id
WHERE kn.base_node_id = (
    SELECT id FROM nodes WHERE ukey = 'VERSE:1:1'
);
```

---

## Performance Considerations

### Indexes Required for Fast Queries

1. **Node lookups by string**: `idx_nodes_ukey` (UNIQUE)
2. **Graph traversal**: `idx_edges_source`, `idx_edges_target`
3. **Knowledge node queries**: `idx_knowledge_nodes_axis`, `idx_knowledge_nodes_base`
4. **Content queries**: `idx_verses_location`

### Expected Performance

- **Node lookup by string**: O(log n) via B-tree index
- **Node lookup by integer**: O(1) via RowID
- **Edge traversal**: O(k) where k = edge count (indexed scan)
- **Knowledge node listing**: O(n) table scan (small table)

### CBOR Serialization

After generation, the entire graph is serialized to CBOR binary format:
- Location: `research_and_dev/iqrah-knowledge-graph2/output/knowledge_graph_1_3.cbor`
- Format: MessagePack-like binary encoding
- Size: ~500KB for chapters 1-3 (vs ~2MB JSON)

---

## References

- [Enum Mappings](../reference/enum-mappings.md) - NodeType, KnowledgeAxis, EdgeType
- [Rust Implementation Guide](rust-implementation-guide.md) - Repository layer
- [Python Generator Guide](python-generator-guide.md) - Two-phase generation
- [Validation Checklist](../reference/validation-checklist.md) - Post-implementation checks
