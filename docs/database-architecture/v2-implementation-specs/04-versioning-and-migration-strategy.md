# Versioning and Migration Strategy

**Last Updated:** 2025-11-17
**Status:** Implementation Ready
**Priority:** P0 (Required before production)

## Context

Two critical versioning concerns:

1. **Content DB Schema Version** - Track content.db structure changes for feature gating and migrations
2. **Knowledge Graph Evolution** - Handle graph schema changes without losing user progress

Currently, neither has a formal migration strategy, creating risk for production deployment.

## Goal

Implement **version tracking and migration strategy** for:
- Content database schema evolution
- Knowledge graph structure changes
- User progress preservation across updates

## Part 1: Content Database Versioning

### Schema Version Table

**Purpose:** Track content.db schema version for runtime feature gating and migration detection.

```sql
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO schema_version (version) VALUES (2);  -- v2 = purist schema
```

### Version History

| Version | Date | Description | Breaking Changes |
|---------|------|-------------|------------------|
| 1 | 2024-11-16 | Generic nodes table | Initial schema |
| 2 | 2025-11-17 | Purist relational schema | Removed nodes table, added domain tables |

### Reading Version at Runtime

**Rust:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs
pub async fn get_content_schema_version(pool: &SqlitePool) -> Result<i32> {
    sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_one(pool)
        .await
        .map_err(|e| Error::SchemaVersionMissing(e.to_string()))
}
```

**Usage:**
```rust
// Feature gating based on schema version
let version = get_content_schema_version(&pool).await?;

match version {
    1 => {
        // Use old node-based queries
        content_repo.get_node(node_id).await?
    }
    2 => {
        // Use new domain-specific queries
        content_repo.get_verse(verse_key).await?
    }
    _ => return Err(Error::UnsupportedSchemaVersion(version)),
}
```

**Validation on Startup:**
```rust
// rust/crates/iqrah-storage/src/content/mod.rs
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    // Run migrations
    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    // Validate version
    let version = get_content_schema_version(&pool).await?;
    const EXPECTED_VERSION: i32 = 2;

    if version != EXPECTED_VERSION {
        return Err(Error::SchemaVersionMismatch {
            expected: EXPECTED_VERSION,
            found: version,
        });
    }

    Ok(pool)
}
```

### Migration Between Schema Versions

**v1 → v2 Migration Strategy:**

**Option A: Rebuild (Recommended)**

**Rationale:** v1 to v2 changes are so extensive (generic nodes → domain tables) that rebuilding is simpler and safer.

**Steps:**
1. Generate new content.db from Python with v2 schema
2. Ship new content.db with app update
3. User DB unaffected (tracks separately via content keys)
4. No complex SQL migration

**Pros:**
- Clean slate
- No migration bugs
- Simpler to reason about

**Cons:**
- Cannot preserve content.db if user has modified it (but content.db is read-only)

**Option B: SQL Migration**

**Only needed if:**
- Content.db has user-added data (unlikely - read-only)
- Want to preserve exact timestamps, etc.

**Migration SQL:**
```sql
-- 20241117000001_migrate_v1_to_v2.sql
BEGIN TRANSACTION;

-- 1. Create v2 tables (from 01-content-schema-v2-purist.md)
-- ... (all CREATE TABLE statements)

-- 2. Migrate data from v1 to v2
INSERT INTO chapters (chapter_number, name_arabic, name_transliteration, name_translation, verse_count)
SELECT
    CAST(SUBSTR(id, 9) AS INTEGER) as chapter_number,
    -- Extract metadata from nodes table (if stored)
    -- ... complex extraction logic
FROM nodes
WHERE node_type = 'chapter';

-- ... similar for verses, words, etc.

-- 3. Drop v1 tables
DROP TABLE nodes;
DROP TABLE edges;  -- Keep if graph still uses edges
DROP TABLE quran_text;

-- 4. Update version
UPDATE schema_version SET version = 2;

COMMIT;
```

**Effort:** 2-3 days + high risk of bugs

**Verdict:** Use Option A (rebuild) for v1→v2.

### Updating Version in Migrations

**When creating a new migration:**

```sql
-- migrations_content/20241117000002_add_new_feature.sql

-- Add new feature (e.g., tafsir table)
CREATE TABLE tafsir (
    verse_key TEXT NOT NULL,
    scholar TEXT NOT NULL,
    tafsir TEXT NOT NULL,
    PRIMARY KEY (verse_key, scholar),
    FOREIGN KEY (verse_key) REFERENCES verses(verse_key)
) STRICT;

-- Increment version
UPDATE schema_version SET version = 3;
```

**Important:** Always update version number when schema structure changes.

## Part 2: Knowledge Graph Migration Strategy

### The Challenge

Knowledge graph is generated in Python, exported as CBOR, imported into Rust.

**Problem:** If node IDs change between graph versions, user progress is lost.

**Example:**
```
Version 1: Node ID = "WORD_INSTANCE:1:1:3"
Version 2: Node ID changes to "WORD:1:1:3" (type rename)

User has memory state for "WORD_INSTANCE:1:1:3"
After update, that node doesn't exist → progress lost
```

### Solution: ID Stability Guarantee

**Design Decision:** Commit to **NEVER changing node IDs** once released to production.

**Guarantee:**
1. Node IDs are **immutable** after first production release
2. Only **ADD** new nodes/edges, never modify existing IDs
3. If schema change is needed, **deprecate** old IDs and **add** new IDs alongside

### Enforcement: Python Validation Script

**Location:** `research_and_dev/iqrah-knowledge-graph2/validate_stability.py`

```python
#!/usr/bin/env python3
"""Validate that new graph version doesn't break ID stability."""

import sys
from pathlib import Path
from typing import Set

def load_graph_node_ids(cbor_path: Path) -> Set[str]:
    """Extract all node IDs from a CBOR graph file."""
    import cbor2

    with open(cbor_path, 'rb') as f:
        records = cbor2.load(f)

    node_ids = set()
    for record in records:
        if record.get('type') == 'node':
            node_ids.add(record['id'])

    return node_ids

def validate_id_stability(old_graph_path: Path, new_graph_path: Path) -> bool:
    """Ensure all old node IDs still exist in new graph."""
    old_ids = load_graph_node_ids(old_graph_path)
    new_ids = load_graph_node_ids(new_graph_path)

    missing_ids = old_ids - new_ids
    if missing_ids:
        print("ERROR: Node IDs removed in new graph version!")
        print(f"Missing IDs count: {len(missing_ids)}")
        print("Sample missing IDs:")
        for node_id in list(missing_ids)[:10]:
            print(f"  - {node_id}")
        return False

    added_ids = new_ids - old_ids
    print(f"✓ ID stability validated")
    print(f"  Old graph: {len(old_ids)} nodes")
    print(f"  New graph: {len(new_ids)} nodes")
    print(f"  Added: {len(added_ids)} new nodes")
    return True

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: validate_stability.py <old_graph.cbor> <new_graph.cbor>")
        sys.exit(1)

    old_path = Path(sys.argv[1])
    new_path = Path(sys.argv[2])

    if not validate_id_stability(old_path, new_path):
        print("\n⚠️ FAILED: Breaking changes detected!")
        print("User progress will be lost if this graph is released.")
        sys.exit(1)

    print("\n✓ PASSED: Graph update is safe")
    sys.exit(0)
```

**Integration into CI/CD:**

```yaml
# .github/workflows/validate-graph.yml
name: Validate Graph Stability

on:
  pull_request:
    paths:
      - 'research_and_dev/iqrah-knowledge-graph2/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 2  # Need previous commit

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: pip install cbor2

      - name: Build old graph
        run: |
          git checkout HEAD~1
          cd research_and_dev/iqrah-knowledge-graph2
          python -m iqrah.build_graph --output /tmp/old_graph.cbor

      - name: Build new graph
        run: |
          git checkout -
          cd research_and_dev/iqrah-knowledge-graph2
          python -m iqrah.build_graph --output /tmp/new_graph.cbor

      - name: Validate stability
        run: |
          cd research_and_dev/iqrah-knowledge-graph2
          python validate_stability.py /tmp/old_graph.cbor /tmp/new_graph.cbor
```

### Graph Version Tracking

**In Python graph builder:**

```python
# research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py

GRAPH_VERSION = 1  # Increment when adding new node types or major structure changes

class GraphBuilder:
    def __init__(self):
        self.metadata = {
            'version': GRAPH_VERSION,
            'generated_at': datetime.now().isoformat(),
            'node_count': 0,
            'edge_count': 0,
        }

    def export_cbor(self, output_path: Path):
        """Export graph with version metadata."""
        with open(output_path, 'wb') as f:
            # First record: metadata
            cbor2.dump({'type': 'metadata', 'data': self.metadata}, f)

            # Remaining records: nodes and edges
            for node in self.nodes:
                cbor2.dump({'type': 'node', 'id': node.id, ...}, f)
            for edge in self.edges:
                cbor2.dump({'type': 'edge', ...}, f)
```

**In Rust CBOR importer:**

```rust
// rust/crates/iqrah-core/src/cbor_import.rs
pub async fn import_cbor_graph_from_bytes(
    cbor_bytes: &[u8],
    content_repo: &dyn ContentRepository,
) -> Result<ImportStats> {
    let mut deserializer = Deserializer::from_reader(cbor_bytes);

    // First record: metadata
    let metadata: CborMetadata = serde::Deserialize::deserialize(&mut deserializer)?;

    // Validate version
    const EXPECTED_GRAPH_VERSION: i32 = 1;
    if metadata.version != EXPECTED_GRAPH_VERSION {
        return Err(Error::GraphVersionMismatch {
            expected: EXPECTED_GRAPH_VERSION,
            found: metadata.version,
        });
    }

    // Import nodes and edges...
}
```

### Handling Unavoidable Breaking Changes

**If** node IDs absolutely must change (rare):

**Step 1: Create Migration Mapping**

```python
# migration_v1_to_v2.json
{
  "version_from": 1,
  "version_to": 2,
  "node_id_mappings": [
    {"old": "WORD_INSTANCE:1:1:1", "new": "WORD:1:1:1"},
    {"old": "WORD_INSTANCE:1:1:2", "new": "WORD:1:1:2"},
    ...
  ]
}
```

**Step 2: Store Mapping in User DB**

```sql
-- In user.db
CREATE TABLE node_id_migrations (
    old_node_id TEXT NOT NULL,
    new_node_id TEXT NOT NULL,
    migration_version INTEGER NOT NULL,
    PRIMARY KEY (old_node_id, migration_version)
) STRICT;
```

**Step 3: Apply Migration**

```rust
pub async fn migrate_user_data_for_graph_update(
    user_repo: &dyn UserRepository,
    mapping_path: &str,
) -> Result<()> {
    // Load mapping
    let mapping: GraphMigrationMapping = serde_json::from_str(
        &std::fs::read_to_string(mapping_path)?
    )?;

    // Begin transaction
    let mut tx = user_repo.begin_transaction().await?;

    // Update memory states
    for (old_id, new_id) in mapping.node_id_mappings {
        sqlx::query(
            "UPDATE user_memory_states
             SET content_key = ?
             WHERE content_key = ?"
        )
        .bind(&new_id)
        .bind(&old_id)
        .execute(&mut *tx)
        .await?;

        // Log migration
        sqlx::query(
            "INSERT INTO node_id_migrations (old_node_id, new_node_id, migration_version)
             VALUES (?, ?, ?)"
        )
        .bind(&old_id)
        .bind(&new_id)
        .bind(mapping.version_to)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
```

**Effort:** 1-2 days per breaking change

**Verdict:** Use ID stability to avoid this complexity.

## Part 3: Content Keys in User DB

### Current Schema (v1)

```sql
-- User DB references graph node IDs
user_memory_states (
    user_id TEXT,
    node_id TEXT,  -- e.g., "WORD_INSTANCE:1:1:3"
    ...
)
```

### v2 Schema (Purist Approach)

**Rename to `content_key` for clarity:**

```sql
-- User DB references content keys
user_memory_states (
    user_id TEXT,
    content_key TEXT,  -- verse_key ("1:1") OR word_id (123) OR chapter_number (1)
    content_type TEXT CHECK (content_type IN ('chapter', 'verse', 'word')),
    ...
)
```

**Why `content_type`?**
- Makes queries clearer
- Enables type-specific indexes
- Documents what kind of content is being tracked

**Migration:**
```sql
-- migrations_user/20241117000001_content_keys.sql
ALTER TABLE user_memory_states RENAME COLUMN node_id TO content_key;
ALTER TABLE user_memory_states ADD COLUMN content_type TEXT;

-- Infer type from key format
UPDATE user_memory_states
SET content_type = CASE
    WHEN content_key LIKE 'WORD_INSTANCE:%' THEN 'word'
    WHEN content_key LIKE 'VERSE:%' THEN 'verse'
    WHEN content_key LIKE 'CHAPTER:%' THEN 'chapter'
    ELSE 'unknown'
END;

-- Extract actual content keys (v2 format)
-- WORD_INSTANCE:1:1:3 → need to lookup word_id from content.db
-- This requires application-level migration (can't do cross-DB in SQL)
```

**Application-Level Migration:**
```rust
pub async fn migrate_node_ids_to_content_keys(
    user_repo: &dyn UserRepository,
    content_repo: &dyn ContentRepository,
) -> Result<()> {
    let states = user_repo.get_all_memory_states().await?;

    for state in states {
        let content_key = match parse_node_id(&state.node_id) {
            NodeId::WordInstance { verse_key, position } => {
                // Lookup word_id from content.db
                let word = content_repo
                    .get_word_by_verse_and_position(&verse_key, position)
                    .await?;
                format!("word:{}", word.id)
            }
            NodeId::Verse { verse_key } => {
                format!("verse:{}", verse_key)
            }
            NodeId::Chapter { chapter_number } => {
                format!("chapter:{}", chapter_number)
            }
            _ => continue,  // Skip unknown types
        };

        user_repo.update_content_key(&state.user_id, &state.node_id, &content_key).await?;
    }

    Ok(())
}
```

## Implementation Steps

### Step 1: Add schema_version to Content DB

**File:** `rust/crates/iqrah-storage/migrations_content/20241117000001_content_schema_v2_purist.sql`

**Add at top:**
```sql
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO schema_version (version) VALUES (2);
```

### Step 2: Add Version Read Function

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`

**Add:**
```rust
pub async fn get_content_schema_version(pool: &SqlitePool) -> Result<i32> {
    sqlx::query_scalar("SELECT version FROM schema_version")
        .fetch_one(pool)
        .await
        .map_err(|_| Error::SchemaVersionMissing)
}
```

### Step 3: Validate Version on Startup

**Update `init_content_db`:**
```rust
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_content").run(&pool).await?;

    let version = get_content_schema_version(&pool).await?;
    if version != 2 {
        return Err(Error::SchemaVersionMismatch { expected: 2, found: version });
    }

    Ok(pool)
}
```

### Step 4: Create Stability Validation Script

**File:** `research_and_dev/iqrah-knowledge-graph2/validate_stability.py`

**Tasks:**
1. Copy script from "Enforcement" section above
2. Test with current graph
3. Add to CI/CD pipeline

### Step 5: Add Graph Version to CBOR

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py`

**Tasks:**
1. Add `GRAPH_VERSION` constant
2. Export version in CBOR metadata
3. Update Rust importer to read and validate version

### Step 6: Migrate User DB to content_key

**File:** `rust/crates/iqrah-storage/migrations_user/20241117000001_content_keys.sql`

**Tasks:**
1. Rename column
2. Add content_type column
3. Create application-level migration function
4. Run migration on app update

## Testing

### Schema Version Tests

```rust
#[tokio::test]
async fn test_schema_version_read() {
    let pool = create_test_content_db().await;
    let version = get_content_schema_version(&pool).await.unwrap();
    assert_eq!(version, 2);
}

#[tokio::test]
async fn test_version_mismatch_error() {
    // Create DB with wrong version
    let pool = create_db_with_version(999).await;
    let result = init_content_db_with_pool(pool).await;
    assert!(matches!(result, Err(Error::SchemaVersionMismatch { .. })));
}
```

### Graph Stability Tests

```python
# test_stability.py
def test_no_nodes_removed():
    old_graph = build_graph_v1()
    new_graph = build_graph_v2()

    old_ids = {n.id for n in old_graph.nodes}
    new_ids = {n.id for n in new_graph.nodes}

    assert old_ids.issubset(new_ids), "Nodes were removed!"

def test_only_nodes_added():
    old_graph = build_graph_v1()
    new_graph = build_graph_v2()

    old_count = len(old_graph.nodes)
    new_count = len(new_graph.nodes)

    assert new_count >= old_count, "Node count decreased!"
```

## Validation Checklist

- [ ] `schema_version` table created in content.db
- [ ] Version read function implemented
- [ ] Version validation on startup implemented
- [ ] Graph stability validation script created
- [ ] CI/CD integration for graph validation
- [ ] Graph version exported in CBOR metadata
- [ ] Rust importer validates graph version
- [ ] User DB migration to content_key completed
- [ ] Application-level migration function tested
- [ ] Tests pass for version validation

## References

- [01-content-schema-v2-purist.md](01-content-schema-v2-purist.md) - Content schema details
- [03-knowledge-graph.md](../03-knowledge-graph.md) - Graph structure (old audit)

---

**Status:** Ready for implementation
**Estimated Effort:**
- Part 1 (Schema Versioning): 4-6 hours
- Part 2 (Graph Stability): 1 day
- Part 3 (User DB Migration): 1 day
**Total:** 2-3 days
