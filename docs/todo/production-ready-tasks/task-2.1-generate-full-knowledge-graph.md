# Task 2.1: Generate Full Knowledge Graph with Axis Nodes

## Metadata
- **Priority:** P0 (Critical - Core Feature)
- **Estimated Effort:** 2-3 days
- **Dependencies:** Task 1.1 (Architecture doc for ID format reference)
- **Agent Type:** Implementation (Python + SQL)
- **Parallelizable:** No (blocks Phase 2 tasks 2.2, 2.3, 2.4)

## Goal

Generate a complete knowledge graph migration file that includes ALL 6 knowledge axis types (memorization, translation, tafseer, tajweed, contextual_memorization, meaning) with proper nodes, edges, and PageRank scores for chapters 1-3.

## Context

**Current State:** The Rust code for knowledge axis is FULLY implemented, but the data is missing:
- ✅ Domain models (`KnowledgeAxis` enum, `KnowledgeNode` struct)
- ✅ Exercise routing by axis
- ✅ Session filtering by axis
- ❌ **Migration file only has content nodes, NO knowledge nodes**

**Current Migration:** `migrations_content/20241118000001_knowledge_graph_chapters_1_3.sql`
- Contains: Verse IDs like `"VERSE:1:1"`, `"VERSE:2:5"` (content nodes)
- Missing: Knowledge nodes like `"VERSE:1:1:memorization"`, `"VERSE:1:1:translation"`

**Why This Matters:**
Knowledge axis is a CORE FEATURE. Without the data:
- Session service returns empty results when filtering by axis
- Users can't practice memorization separately from translation
- The sophisticated axis-specific exercise system is unused
- Cross-axis propagation (translation helps memorization) doesn't work

**Python Generator:** The code EXISTS in `knowledge_builder.py` but hasn't been run to generate the current migration.

## Current State

**Python R&D Project:**
- **Location:** `research_and_dev/iqrah-knowledge-graph2/`
- **Builder:** `src/iqrah/graph/knowledge_builder.py` (lines 139-196)
- **Exporter:** `score_and_extract.py`

**Current Migration File:**
- **Path:** `rust/crates/iqrah-storage/migrations_content/20241118000001_knowledge_graph_chapters_1_3.sql`
- **Size:** ~126KB
- **Content:** 493 verses, dependency edges, PageRank scores
- **Missing:** Knowledge nodes and knowledge edges

**Rust Expectations:**
- `rust/crates/iqrah-core/src/domain/models.rs` (lines 54-147) - Expects 6 axis types
- `rust/crates/iqrah-core/src/exercises/service.rs` (lines 84-146) - Routes by axis
- `rust/crates/iqrah-core/src/services/session_service.rs` (lines 106-120) - Filters by axis

## Target State

### New Migration File Structure

**File:** `rust/crates/iqrah-storage/migrations_content/20241124000002_knowledge_graph_full_axis.sql`

**Contents:**
1. **Content Nodes** (already exist):
   - Chapters: `CHAPTER:1`, `CHAPTER:2`, `CHAPTER:3`
   - Verses: `VERSE:1:1`, `VERSE:1:2`, ..., `VERSE:3:200` (493 verses total)
   - Words: `WORD_INSTANCE:1:1:1`, `WORD_INSTANCE:1:1:2`, ...

2. **Knowledge Nodes** (NEW):
   - Verse memorization: `VERSE:1:1:memorization`, `VERSE:1:2:memorization`, ... (493 nodes)
   - Verse translation: `VERSE:1:1:translation`, `VERSE:1:2:translation`, ... (493 nodes)
   - Verse tafsir: `VERSE:1:1:tafsir`, ... (493 nodes)
   - Verse tajweed: `VERSE:1:1:tajweed`, ... (493 nodes)
   - Word memorization: `WORD_INSTANCE:1:1:1:memorization`, ...
   - Word translation: `WORD_INSTANCE:1:1:1:translation`, ...
   - (Estimate: ~2000-3000 knowledge nodes total)

3. **Dependency Edges** (already exist):
   - Sequential: Verse N → Verse N+1
   - Hierarchical: Chapter → Verse, Verse → Word

4. **Knowledge Edges** (NEW):
   - Sequential: `VERSE:1:1:memorization` → `VERSE:1:2:memorization`
   - Cross-axis: `VERSE:1:1:translation` → `VERSE:1:1:memorization` (translation helps memorization)
   - Contextual: `WORD_INSTANCE:1:1:1:memorization` → `VERSE:1:1:memorization`

5. **Node Metadata** (updated):
   - PageRank scores for ALL nodes (content + knowledge)
   - Foundational score, influence score, difficulty score

6. **Goals** (updated):
   - Goal: `memorization:chapters-1-3` → links to memorization nodes
   - Goal: `translation:chapters-1-3` → links to translation nodes

### Expected Size

- Current: 126KB
- With knowledge nodes: ~300-500KB (still reasonable for SQLite migration)

## Implementation Steps

### Step 1: Understand Current Python Code (1 hour)

**Read:**
- `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py`
- `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py`
- `research_and_dev/iqrah-knowledge-graph2/score_and_extract.py`

**Verify:**
- `build_memorization_edges()` function exists
- Knowledge axis enum matches Rust (6 types)
- Edge types include knowledge edges

### Step 2: Configure Graph Builder for Full Axis (1 hour)

**File:** `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py`

Ensure the builder generates all 6 axes. Look for:

```python
def build_knowledge_graph(chapters_range=(1, 3)):
    """Build complete knowledge graph with all axes."""

    # Base dependency graph
    graph = build_dependency_graph(chapters_range)

    # Add knowledge nodes and edges for each axis
    for axis in KnowledgeAxis:
        add_axis_nodes(graph, axis, chapters_range)
        add_axis_edges(graph, axis)

    # Add cross-axis edges (translation → memorization, etc.)
    add_cross_axis_edges(graph)

    return graph
```

**If this doesn't exist**, you may need to implement it based on existing code patterns.

### Step 3: Build Knowledge Graph with All Axes (1-2 hours)

**Command:**
```bash
cd research_and_dev/iqrah-knowledge-graph2

# Build complete knowledge graph with all axes and scoring
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology/quran-morphology-v0.5.csv \
    --preset full \
    --chapters "1-3" \
    -o output/knowledge_graph_full_axis.cbor.zst
```

**What this does:**
- Builds dependency graph from scratch (chapters, verses, words, lemmas, roots)
- Adds knowledge edges for all 6 axes (memorization, translation, tafsir, tajweed, contextual_memorization, meaning)
- Computes PageRank scores (integrated, no separate scoring step needed)
- Exports to CBOR format with Zstandard compression

**Verify output:**
```bash
# Check file was created
ls -lh output/knowledge_graph_full_axis.cbor.zst

# Inspect graph structure
python inspect_graph.py output/knowledge_graph_full_axis.cbor.zst
```

**Expected output:**
```
File size: 5.2 MB
Version: 2
Format: structure_only
Nodes: 11,234
Edges: 45,678

Node types:
  chapter: 3
  verse: 493
  word: 2,145
  word_instance: 6,234
  lemma: 1,523
  root: 836

Sample nodes with knowledge axes:
  - VERSE:1:1:memorization
  - VERSE:1:1:translation
  - WORD_INSTANCE:1:1:1:memorization
```

### Step 4: Export Graph to SQL Migration (1 hour)

**Note:** The `score_and_extract.py` script is for the OLD workflow that generates SQL directly from GraphML. For the new CBOR-based workflow, we need a different approach.

**Option A: Use existing export script (if it supports CBOR input):**

Check if `score_and_extract.py` can read CBOR files:

```bash
python score_and_extract.py --help
```

If it supports CBOR:
```bash
python score_and_extract.py \
    output/knowledge_graph_full_axis.cbor.zst \
    > ../../rust/crates/iqrah-storage/migrations_content/20241124000002_knowledge_graph_full_axis.sql
```

**Option B: Create new CBOR-to-SQL exporter:**

If `score_and_extract.py` only works with GraphML, you'll need to either:
1. Export to GraphML format during build (use `--format both`)
2. Create a new script to convert CBOR → SQL

**Recommended approach:**

```bash
# Build with both CBOR and GraphML output
python -m iqrah_cli build knowledge-graph \
    --from-scratch \
    --morphology ../data/morphology/quran-morphology-v0.5.csv \
    --preset full \
    --chapters "1-3" \
    --format both \
    -o output/knowledge_graph_full_axis.cbor.zst

# This creates:
# - output/knowledge_graph_full_axis.cbor.zst (for Rust import)
# - output/knowledge_graph_full_axis.graphml (for SQL generation)

# Generate SQL from GraphML
python score_and_extract.py output/knowledge_graph_full_axis.graphml
```

**Verify SQL structure:**
```sql
-- Should contain:

-- Knowledge nodes in node_metadata
INSERT INTO node_metadata (node_id, key, value) VALUES
    ('VERSE:1:1:memorization', 'foundational_score', 0.0123),
    ('VERSE:1:1:translation', 'foundational_score', 0.0098),
    ...;

-- Knowledge edges
INSERT INTO edges (source_id, target_id, edge_type, distribution_type, distribution_param1, distribution_param2) VALUES
    ('VERSE:1:1:memorization', 'VERSE:1:2:memorization', 1, 0, 0.8, 0.0),  -- Sequential
    ('VERSE:1:1:translation', 'VERSE:1:1:memorization', 1, 0, 0.3, 0.0),  -- Cross-axis
    ...;

-- Updated goals with axis-specific nodes
INSERT INTO node_goals (goal_id, node_id) VALUES
    ('memorization:chapters-1-3', 'VERSE:1:1:memorization'),
    ('memorization:chapters-1-3', 'VERSE:1:2:memorization'),
    ...;
```

### Step 6: Verify Migration File (30 min)

**Checks:**
1. **File size:** 300-500KB (reasonable)
2. **Node count:** Query `SELECT COUNT(DISTINCT node_id) FROM node_metadata` after import
   - Should be ~2500-3500 (content + knowledge nodes)
3. **Edge types:** Query `SELECT edge_type, COUNT(*) FROM edges GROUP BY edge_type`
   - Type 0 (dependency): ~500-1000 edges
   - Type 1 (knowledge): ~2000-3000 edges
4. **Axis coverage:** Search for all 6 axes:
   ```bash
   grep -c ":memorization" migration.sql
   grep -c ":translation" migration.sql
   grep -c ":tafsir" migration.sql
   grep -c ":tajweed" migration.sql
   grep -c ":contextual_memorization" migration.sql
   grep -c ":meaning" migration.sql
   ```
   All should return > 0.

### Step 7: Test Import in Rust (1 hour)

**Commands:**
```bash
cd rust

# Create test database
rm -f /tmp/test_content.db
cargo run --bin iqrah-cli -- init --content-db /tmp/test_content.db

# Migrations should run automatically
# Verify data imported

sqlite3 /tmp/test_content.db "SELECT COUNT(DISTINCT node_id) FROM node_metadata WHERE node_id LIKE '%:memorization'"
# Should return > 400 (one per verse)

sqlite3 /tmp/test_content.db "SELECT COUNT(*) FROM edges WHERE edge_type = 1"
# Should return > 2000 (knowledge edges)
```

### Step 8: Test Rust Code End-to-End (1 hour)

**Commands:**
```bash
cd rust

# Test session generation with axis filtering
cargo run --bin iqrah-cli -- schedule \
    --goal memorization:chapters-1-3 \
    --axis memorization \
    --limit 5

# Should return 5 session items with node IDs ending in ":memorization"

# Test translation axis
cargo run --bin iqrah-cli -- schedule \
    --goal memorization:chapters-1-3 \
    --axis translation \
    --limit 5

# Should return 5 session items with node IDs ending in ":translation"
```

**Expected output:**
```
Session items (memorization axis):
1. Node: 1:1:memorization, Type: Knowledge, Score: 0.0649
2. Node: 1:2:memorization, Type: Knowledge, Score: 0.0523
...
```

## Verification Plan

### Python Tests

```bash
cd research_and_dev/iqrah-knowledge-graph2

# Test graph builder
pytest tests/test_knowledge_builder.py -v

# Test scoring
pytest tests/test_scoring.py -v
```

- [ ] Graph builder creates knowledge nodes
- [ ] All 6 axes present in output
- [ ] PageRank scores assigned to all nodes
- [ ] Cross-axis edges exist

### SQL Validation

```bash
cd rust/crates/iqrah-storage

# Import migration
sqlite3 test.db < migrations_content/20241124000002_knowledge_graph_full_axis.sql

# Count nodes by type
sqlite3 test.db "SELECT COUNT(*) FROM node_metadata WHERE node_id LIKE '%:memorization'"
# Expected: ~493 (one per verse)

sqlite3 test.db "SELECT COUNT(*) FROM node_metadata WHERE node_id LIKE '%:translation'"
# Expected: ~493

# Count knowledge edges
sqlite3 test.db "SELECT COUNT(*) FROM edges WHERE edge_type = 1"
# Expected: > 2000

# Verify goals updated
sqlite3 test.db "SELECT goal_id, COUNT(*) FROM node_goals GROUP BY goal_id"
# Expected: memorization:chapters-1-3 has ~493 nodes
```

### Rust Integration Tests

```bash
cd rust

# Run all tests (should still pass)
cargo test --all-features

# Run specific integration test
cargo test --test knowledge_axis_test -- --nocapture
```

- [ ] Knowledge axis parsing works
- [ ] Session generation returns knowledge nodes
- [ ] Exercise generation routes by axis
- [ ] All existing tests still pass

### CLI End-to-End Test

```bash
# Generate sessions for each main verse-level axis
# (contextual_memorization and meaning are word-level only)
for axis in memorization translation tafsir tajweed; do
    echo "Testing axis: $axis"
    cargo run --bin iqrah-cli -- schedule \
        --goal memorization:chapters-1-3 \
        --axis $axis \
        --limit 3
done
```

- [ ] All 4 main verse-level axes return results
- [ ] Node IDs end with correct axis suffix
- [ ] No errors or panics

**Note on Axis Distribution:**
The 6 knowledge axes are distributed across two granularity levels:
- **Verse-level axes (4):** memorization, translation, tafsir, tajweed
  - Applied to all 493 verses in chapters 1-3
  - User practices these at verse granularity (e.g., `VERSE:1:1:memorization`)
- **Word-level axes (2):** contextual_memorization, meaning
  - Applied to individual word instances
  - User practices these at word granularity (e.g., `WORD_INSTANCE:1:1:3:contextual_memorization`)

## Scope Limits & Safeguards

### ✅ MUST DO

- Generate knowledge nodes for all 6 axes
- Include both verse-level and word-level knowledge nodes
- Add knowledge edges (sequential + cross-axis)
- Compute PageRank scores for all nodes
- Update goals to link to axis-specific nodes
- Generate valid SQL migration file
- Test import in Rust
- Verify CLI commands work with axis filtering

### ❌ DO NOT

- Change Rust code (except if bug discovered during testing)
- Modify existing migration files (create new one)
- Change node ID formats (follow Task 1.1 specification)
- Add new features beyond knowledge axis (scope creep)
- Touch Flutter/UI code

### ⚠️ If Uncertain

- If Python CLI doesn't exist → check `src/iqrah/cli/` or run builder script directly
- If graph is too large (>1MB SQL) → reduce to chapters 1-2 temporarily
- If PageRank fails → check for disconnected components in graph
- If SQL syntax errors → validate with `sqlite3 :memory: < file.sql`
- If Rust tests fail → check that node IDs match expected format exactly

## Success Criteria

- [ ] New migration file created: `20241124000002_knowledge_graph_full_axis.sql`
- [ ] File contains knowledge nodes (verify with `grep ":memorization"`)
- [ ] All 6 axes present (memorization, translation, tafsir, tajweed, contextual_memorization, meaning)
- [ ] Knowledge edges present (EdgeType::Knowledge = 1)
- [ ] PageRank scores for all nodes
- [ ] Migration imports successfully into SQLite
- [ ] Node count: 2500-3500 (content + knowledge)
- [ ] Edge count: 2000-4000 (dependency + knowledge)
- [ ] CLI test: `iqrah schedule --axis memorization` returns results
- [ ] CLI test: `iqrah schedule --axis translation` returns results
- [ ] All Rust tests pass
- [ ] No warnings or errors during build

## Related Files

**Python Files (Read/Modify):**
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py`
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge.py`
- `/research_and_dev/iqrah-knowledge-graph2/score_and_extract.py`

**Generated SQL File:**
- `/rust/crates/iqrah-storage/migrations_content/20241124000002_knowledge_graph_full_axis.sql` (NEW)

**Verification Files (No Changes):**
- `/rust/crates/iqrah-core/src/domain/models.rs` - Check axis enum matches
- `/rust/crates/iqrah-core/src/exercises/service.rs` - Verify exercise routing
- `/rust/crates/iqrah-core/src/services/session_service.rs` - Verify session filtering

## Notes

### Knowledge Axis Design

**Per Verse (493 verses):**
- `1:1:memorization` - Can you recite this verse?
- `1:1:translation` - Do you understand the meaning?
- `1:1:tafsir` - Do you understand the context/commentary?
- `1:1:tajweed` - Can you recite with proper pronunciation?

**Per Word (thousands):**
- `WORD_INSTANCE:1:1:3:memorization` - Can you recall this word?
- `WORD_INSTANCE:1:1:3:translation` - Do you know the meaning?

### Cross-Axis Synergies

**Designed edges:**
- Translation → Memorization (understanding helps recall)
- Tafsir → Translation (context helps understanding)
- Contextual memorization → Memorization (word recall helps verse recall)

### Graph Statistics (Estimate)

**For chapters 1-3 (493 verses, ~6000 words):**
- Content nodes: ~6500
- Knowledge nodes (verse × 4 verse-level axes): ~2000
- Knowledge nodes (word × 2 word-level axes): ~3000
- **Total nodes: ~11,500** (6 knowledge axes total)
- Dependency edges: ~6000
- Knowledge edges: ~10,000
- **Total edges: ~16,000**

**SQL file size:** ~400-600KB (still reasonable)

### Performance Impact

SQLite handles tens of thousands of rows easily. On mobile:
- Index on `node_id`: O(log n) lookups
- Session generation: <20ms even with 10k+ nodes (tested in scheduler v2)

No performance concerns.

### Future Work

This task covers chapters 1-3. Later:
- Task X.X: Generate full Quran (chapters 4-114)
- Estimated: ~200K nodes, ~500K edges, ~20-30MB SQL
- Same process, just larger input range
