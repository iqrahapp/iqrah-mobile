# Knowledge Graph Architecture

**Related Question:** Q3 - How are knowledge graph migrations handled and how is user progression preserved?

## Overview

The Knowledge Graph is a **directed graph structure** representing:
- Hierarchical relationships (chapter → verse → word)
- Dependency edges (verse depends on words being known)
- Knowledge edges (understanding translation helps memorization)
- Spaced repetition metadata

The graph is **generated in Python** and **imported into Rust** via CBOR format.

## Graph Structure

### Node Types

```rust
// rust/crates/iqrah-core/src/domain/models.rs:6-16
pub enum NodeType {
    Root,           // Morphological root (e.g., ROOT:ktb)
    Lemma,          // Lemma (e.g., LEMMA:kataba)
    Word,           // Word form (e.g., WORD:1:1:1)
    WordInstance,   // Specific occurrence (e.g., WORD_INSTANCE:1:1:1)
    Verse,          // Verse (e.g., VERSE:1:1)
    Chapter,        // Chapter (e.g., CHAPTER:1)
    Knowledge,      // Knowledge axis node (e.g., WORD_INSTANCE:1:1:1:memorization)
}
```

### Node ID Conventions

**Python Generation:** [graph/identifiers.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/identifiers.py)

```python
# Lines 35-51
def for_chapter(chapter_number: int) -> str:
    return f"CHAPTER:{chapter_number}"

def for_verse(verse: Verse) -> str:
    return f"VERSE:{verse.verse_key}"  # e.g., "VERSE:1:1"

def for_word_instance(word: Word, verse: Verse) -> str:
    return f"WORD_INSTANCE:{verse.verse_key}:{word.position}"  # e.g., "WORD_INSTANCE:1:1:3"

def for_lemma(lemma_id: str) -> str:
    return f"LEMMA:{lemma_id}"

def for_root(root_id: str) -> str:
    return f"ROOT:{root_id}"

def for_knowledge_node(base_node_id: str, axis: KnowledgeAxis) -> str:
    return f"{base_node_id}:{axis.value}"  # e.g., "WORD_INSTANCE:1:1:3:memorization"
```

**Key Insight:** Node IDs are **semantic strings** that encode their position in the Quran. This enables:
- Direct ID construction (next word = increment position)
- Human readability
- Stable references across graph versions (if structure doesn't change)

### Edge Types

```rust
// Stored in edges table
pub struct Edge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub distribution_type: DistributionType,
    pub param1: Option<f64>,
    pub param2: Option<f64>,
}

pub enum EdgeType {
    Dependency = 0,  // Source depends on target (structural)
    Knowledge = 1,   // Source gains knowledge from target (semantic)
}
```

**Dependency Edges:** Represent learning prerequisites.
```
VERSE:1:1 → WORD_INSTANCE:1:1:1  (to know verse, must know word)
WORD_INSTANCE:1:1:1 → WORD:1:1:1  (word instance derives from word form)
```

**Knowledge Edges:** Represent learning synergies.
```
WORD_INSTANCE:1:1:1:translation → WORD_INSTANCE:1:1:1:memorization
(understanding translation helps memorization)
```

### Distribution Types

Energy transfer between nodes is probabilistic:

```rust
pub enum DistributionType {
    Const = 0,    // Fixed value (param1)
    Normal = 1,   // Normal distribution (param1=mean, param2=std)
    Beta = 2,     // Beta distribution (param1=alpha, param2=beta)
}
```

**Usage Examples:**
- **Const:** Word → Verse dependency (always transfer 0.2 energy)
- **Normal:** Adjacent word knowledge (mean=0.3, std=0.1 - some variability)
- **Beta:** Morphological relationships (skewed distributions)

## Graph Generation (Python)

### Build Process

**Location:** [research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/)

**Key Files:**
- `builder.py` - Dependency graph builder
- `knowledge_builder.py` - Knowledge edge builder
- `identifiers.py` - Node ID generation
- `knowledge.py` - Axis validation and semantics

**Build Workflow:**
```python
# 1. Build dependency graph (structural)
builder = GraphBuilder(quran_data)
builder.build_dependency_graph()
# Creates: CHAPTER → VERSE → WORD_INSTANCE hierarchy

# 2. Add knowledge edges (semantic)
knowledge_builder = KnowledgeBuilder(builder.edge_manager)
knowledge_builder.build_knowledge_edges()
# Creates: translation→memorization, word→word knowledge edges

# 3. Export to CBOR
exporter = GraphExporter(builder.nodes, builder.edges)
exporter.export_to_cbor("graph.cbor")
```

### Dependency Graph Structure

**Code:** [graph/builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py)

**Example for Surah Al-Fatihah (Chapter 1):**

```
CHAPTER:1
├─→ VERSE:1:1
│   ├─→ WORD_INSTANCE:1:1:1 (بِسْمِ)
│   ├─→ WORD_INSTANCE:1:1:2 (ٱللَّهِ)
│   ├─→ WORD_INSTANCE:1:1:3 (ٱلرَّحْمَٰنِ)
│   └─→ WORD_INSTANCE:1:1:4 (ٱلرَّحِيمِ)
├─→ VERSE:1:2
│   ├─→ WORD_INSTANCE:1:2:1 (ٱلْحَمْدُ)
│   └─→ ...
└─→ ...
```

**Edge direction:** Parent depends on children (verse mastery requires word mastery).

### Knowledge Graph Structure

**Code:** [graph/knowledge_builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py)

**Hierarchical Knowledge Edges:**
```python
# Lines 161-165: Verse memorization contributes to chapter memorization
self.edge_manager.add_knowledge_edge(
    f"{verse_id}:memorization",
    f"{chapter_id}:memorization",
    Distribution.auto(weight=verse.get_letters_count())
)

# Lines 178-182: Word memorization contributes to verse memorization
self.edge_manager.add_knowledge_edge(
    f"{word_id}:memorization",
    f"{verse_id}:memorization",
    Distribution.auto(weight=word.get_letters_count())
)
```

**Cross-Axis Knowledge Edges:**
```python
# Lines 301-306: Translation understanding helps memorization
self.edge_manager.add_knowledge_edge(
    f"{node_id}:translation",
    f"{node_id}:memorization",
    Distribution.normal(mean=0.4, std=0.15)
)
```

**Adjacent Word Edges:**
```python
# Lines 390-410: Sequential word relationships
self.edge_manager.add_knowledge_edge(
    f"{current_word_id}:contextual_memorization",
    f"{previous_word_id}:contextual_memorization",
    Distribution.normal(mean=0.35, std=0.12)
)
```

## CBOR Import (Rust)

### CBOR Format

**CBOR (Compact Binary Object Representation):** Efficient binary serialization format.

**Why CBOR?**
- Smaller than JSON (50-70% size reduction)
- Faster to parse
- Type-safe binary format
- Supports streaming

**Python Export:**
```python
# graph/exporter.py (conceptual)
import cbor2

data = {
    'nodes': [{'id': 'VERSE:1:1', 'type': 'verse', ...}, ...],
    'edges': [{'source': 'VERSE:1:1', 'target': 'WORD_INSTANCE:1:1:1', ...}, ...]
}

with open('graph.cbor', 'wb') as f:
    cbor2.dump(data, f)
```

### Rust Import Process

**Location:** [rust/crates/iqrah-core/src/cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs) (lines 98-226)

**Import Flow:**

```rust
pub async fn import_cbor_graph_from_bytes(
    cbor_bytes: &[u8],
    content_repo: &dyn ContentRepository,
) -> Result<ImportStats> {
    // 1. Deserialize CBOR
    let records: Vec<CborRecord> = serde_cbor::from_slice(cbor_bytes)?;

    // 2. Batch collect nodes and edges
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for record in records {
        match record {
            CborRecord::Node { id, a } => {
                nodes.push(ImportedNode {
                    id,
                    node_type: a.node_type,
                    metadata: a.metadata,
                });
            }
            CborRecord::Edge { source_id, target_id, edge_type, ... } => {
                edges.push(ImportedEdge {
                    source_id,
                    target_id,
                    edge_type,
                    distribution_type,
                    param1,
                    param2,
                });
            }
        }
    }

    // 3. Batch insert to database
    content_repo.insert_nodes_batch(&nodes).await?;
    content_repo.insert_edges_batch(&edges).await?;

    Ok(ImportStats { nodes_imported: nodes.len(), edges_imported: edges.len() })
}
```

**Batch Insert Implementation:** [content/repository.rs:97-147](../../rust/crates/iqrah-storage/src/content/repository.rs)

```rust
// Batch insert with transaction
let mut tx = self.pool.begin().await?;

for node in nodes {
    query("INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)")
        .bind(&node.id)
        .bind(&node.node_type.to_string())
        .bind(Utc::now().timestamp())
        .execute(&mut *tx)
        .await?;
}

tx.commit().await?;
```

**Key Feature:** `INSERT OR IGNORE` makes imports **idempotent** (safe to re-run).

## Q3: Knowledge Graph Migrations & User Progression

### Current Migration Strategy

**Answer:** There is **NO explicit migration strategy** for knowledge graph schema changes.

**Current Approach:**
1. Python generates new graph → exports new CBOR
2. Rust app ships with new CBOR file
3. On app update, CBOR is re-imported
4. `INSERT OR IGNORE` prevents duplicate nodes/edges

**Limitations:**

| Scenario | Current Behavior | User Impact |
|----------|-----------------|-------------|
| New node added | Node inserted successfully | ✅ No data loss |
| Node ID changed | New node created, old node orphaned | ❌ User progress lost for that node |
| Edge modified | Edge updated (if logic changed) | ⚠️ Energy propagation may change |
| Node deleted | Node remains in DB (orphaned) | ⚠️ User data persists but node unreachable |

### User Progression Preservation

**Storage Location:** User DB, completely separate from Content DB.

**Memory States Keyed by Node ID:**
```rust
// user_memory_states table
PRIMARY KEY (user_id, node_id)
```

**Example:**
```sql
user_id   | node_id                | stability | energy | ...
----------|------------------------|-----------|--------|----
default   | WORD_INSTANCE:1:1:1    | 3.2       | 0.75   | ...
default   | VERSE:1:1              | 5.1       | 0.60   | ...
```

**Preservation Guarantee:**
- ✅ User progress persists across app updates
- ✅ Content DB can be replaced without touching User DB
- ❌ BUT: Progress is tied to node IDs (if node ID changes, progress is lost)

### Migration Scenarios

#### Scenario 1: Add New Chapter
**Example:** Quran initially has chapters 1-10, update adds chapter 11.

**Process:**
1. Python generates new node: `CHAPTER:11`
2. CBOR export includes new node
3. Rust imports: `INSERT OR IGNORE INTO nodes ...`
4. New node available for learning

**User Impact:** ✅ None. Existing progress unaffected.

#### Scenario 2: Fix Verse Numbering
**Example:** Verse numbering error - VERSE:2:10 should be VERSE:2:11.

**Problem:**
- Old node ID: `VERSE:2:10`
- New node ID: `VERSE:2:11`
- User has memory state for `VERSE:2:10`

**Current Behavior:**
- New node `VERSE:2:11` created
- Old memory state for `VERSE:2:10` orphaned
- User loses progress for that verse

**Needed Solution:**
```sql
-- Migration script (not currently implemented)
UPDATE user_memory_states
SET node_id = 'VERSE:2:11'
WHERE node_id = 'VERSE:2:10';
```

#### Scenario 3: Change Knowledge Axis Design
**Example:** Rename `memorization` axis to `recall`.

**Problem:**
- Old: `WORD_INSTANCE:1:1:1:memorization`
- New: `WORD_INSTANCE:1:1:1:recall`

**Current Behavior:** Same as Scenario 2 - progress lost.

### Recommended Migration Strategy

**Option 1: Node ID Stability Guarantee** (Preferred)
- Commit to never changing node IDs once released
- Only add new nodes or edges
- If schema change needed, create new node IDs and deprecate old ones

**Option 2: Explicit Migration Mapping**
```rust
// migrations/content_to_user_mapping.json
{
    "version": 2,
    "node_id_changes": [
        {"old": "VERSE:2:10", "new": "VERSE:2:11"},
        {"old": "WORD_INSTANCE:1:1:1:memorization", "new": "WORD_INSTANCE:1:1:1:recall"}
    ]
}

// Apply during app update
for change in mapping.node_id_changes {
    user_repo.migrate_node_id(change.old, change.new).await?;
}
```

**Option 3: Content Version Tracking**
```sql
-- Add to user DB
CREATE TABLE content_version (
    version INTEGER PRIMARY KEY,
    imported_at INTEGER
);

-- Before import
let current_version = user_repo.get_content_version().await?;
let new_version = cbor_metadata.version;

if new_version > current_version {
    apply_migrations(current_version, new_version).await?;
}
```

## Graph Validation

**Python Side:** Extensive validation during generation.

```python
# graph/knowledge.py:86-125
def _validate_knowledge_node(self, node_id: str):
    *base_parts, axis = node_id.split(":")
    base_node = ":".join(base_parts)

    node_type = self._get_node_type(base_node)

    # Check if axis is allowed for this node type
    if knowledge_axis not in node_type.value:
        raise ValueError(f"Invalid knowledge axis '{axis}' for node type {node_type}")
```

**Rust Side:** Minimal validation during import.
- Type checking via enum deserialization
- Foreign key constraints (edges reference nodes)
- No semantic validation

## File Locations

**Python Generation:**
- Graph Builder: [graph/builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/builder.py)
- Knowledge Builder: [graph/knowledge_builder.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py)
- Identifiers: [graph/identifiers.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/identifiers.py)
- Validation: [graph/knowledge.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py)

**Rust Import:**
- CBOR Import: [cbor_import.rs](../../rust/crates/iqrah-core/src/cbor_import.rs) (lines 98-226)
- Content Repository: [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 97-147)

---

**Navigation:** [← User Database](02-user-database.md) | [Next: Database Interactions →](04-database-interactions.md)
