# Task 1.4: Refactor Repository to Use Node ID Module

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 2 days
- **Dependencies:** Task 1.3 (Node ID utility module must exist)
- **Agent Type:** Implementation + Refactoring
- **Parallelizable:** No (depends on 1.3)

## Goal

Replace ad-hoc string parsing logic in `SqliteContentRepository` with the type-safe node_id utility module, eliminating brittle string manipulation and improving code maintainability.

## Context

Currently, `SqliteContentRepository::get_node()` and related methods use ad-hoc string parsing:

```rust
// Current approach (BRITTLE):
let parts: Vec<&str> = node_id.split(':').collect();
if parts.len() == 2 {
    // Try as verse...
} else if parts.len() == 1 {
    // Try as chapter... or word?
    let num: i64 = parts[0].parse().unwrap();  // UNSAFE!
    if num < 114 {
        // Assume chapter
    } else {
        // Assume word
    }
}
```

**Problems:**
- Integer guessing (if num < 114) is ambiguous and buggy
- Unwraps can panic on malformed IDs
- Parsing logic duplicated across methods
- Hard to maintain and error-prone

**After Task 1.3**, we have clean utilities:
```rust
use iqrah_core::domain::node_id;

let (ch, v) = node_id::parse_verse(id)?;  // Type-safe!
```

This task applies those utilities throughout the repository layer.

## Current State

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

**Methods with String Parsing:**
- `get_node()` (lines ~32-80) - Main parsing logic
- `get_edges_from()` - May construct node IDs
- `get_verse()` - May parse verse keys
- `get_nodes_for_goal()` - Returns node IDs
- Any other methods that build or parse node IDs

**Current Logic:**
```rust
async fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
    // Ad-hoc string splitting
    // Integer guessing
    // Multiple query attempts
    // Fallback logic
}
```

## Target State

**Refactored `get_node()` Using node_id Module:**

```rust
use iqrah_core::domain::node_id;

async fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
    // Detect type first
    let node_type = node_id::node_type(node_id)?;

    match node_type {
        NodeType::Chapter => {
            let num = node_id::parse_chapter(node_id)?;
            self.get_chapter_node(num).await
        }
        NodeType::Verse => {
            let (ch, v) = node_id::parse_verse(node_id)?;
            self.get_verse_node(ch, v).await
        }
        NodeType::Word => {
            let word_id = node_id::parse_word(node_id)?;
            self.get_word_node(word_id).await
        }
        NodeType::WordInstance => {
            let (ch, v, pos) = node_id::parse_word_instance(node_id)?;
            self.get_word_instance_node(ch, v, pos).await
        }
        NodeType::Knowledge => {
            let (base_id, axis) = node_id::parse_knowledge(node_id)?;
            // Knowledge nodes ARE stored in node_metadata with their full ID
            // (e.g., "VERSE:1:1:memorization" is a real node in the database)
            let node = self.get_node_by_id(node_id).await?;
            Ok(node.map(|n| Node {
                id: n.id,
                node_type: NodeType::Knowledge,
                axis: Some(axis),
                base_id: Some(base_id),
                // ... other fields from database
            }))
        }
    }
}
```

**Benefits:**
- No ad-hoc string parsing
- Type-safe extraction of IDs
- Clear error handling
- Maintainable code
- No integer guessing

## Implementation Steps

### Step 1: Add node_id Dependency (10 min)

**File:** `rust/crates/iqrah-storage/Cargo.toml`

Ensure `iqrah-core` is a dependency:
```toml
[dependencies]
iqrah-core = { path = "../iqrah-core" }
# ... other deps
```

### Step 2: Refactor `get_node()` Method (2-3 hours)

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

**Current implementation** (lines ~32-80):
- Read and understand the current logic
- Identify what queries are executed for each node type

**New implementation:**
```rust
use iqrah_core::domain::node_id;
use iqrah_core::domain::models::NodeType;

async fn get_node(&self, node_id: &str) -> Result<Option<Node>> {
    // Use node_id module for type detection
    let node_type = node_id::node_type(node_id)
        .map_err(|e| StorageError::InvalidNodeId {
            node_id: node_id.to_string(),
            reason: e.to_string(),
        })?;

    match node_type {
        NodeType::Chapter => {
            let chapter_num = node_id::parse_chapter(node_id)?;
            // Query: SELECT * FROM chapters WHERE chapter_number = ?
            self.get_chapter_by_number(chapter_num).await
        }

        NodeType::Verse => {
            let (chapter, verse) = node_id::parse_verse(node_id)?;
            let verse_key = format!("{}:{}", chapter, verse);
            // Query: SELECT * FROM verses WHERE verse_key = ?
            self.get_verse_by_key(&verse_key).await
        }

        NodeType::Word => {
            let word_id = node_id::parse_word(node_id)?;
            // Query: SELECT * FROM words WHERE word_id = ?
            self.get_word_by_id(word_id).await
        }

        NodeType::WordInstance => {
            let (chapter, verse, position) = node_id::parse_word_instance(node_id)?;
            let verse_key = format!("{}:{}", chapter, verse);
            // Query: SELECT * FROM words WHERE verse_key = ? AND position = ?
            self.get_word_by_position(&verse_key, position).await
        }

        NodeType::Knowledge => {
            let (base_id, axis) = node_id::parse_knowledge(node_id)?;
            // Knowledge nodes ARE stored in node_metadata table
            // Query: SELECT * FROM node_metadata WHERE node_id = ? (e.g., "VERSE:1:1:memorization")
            self.get_node_metadata(node_id).await.map(|metadata| {
                metadata.map(|meta| Node {
                    id: node_id.to_string(),
                    node_type: NodeType::Knowledge,
                    axis: Some(axis),
                    base_id: Some(base_id),
                    scores: meta.scores,
                    // ... other fields from node_metadata
                })
            })
        }
    }
}
```

**Note:** You may need to extract helper methods like `get_chapter_by_number()`, `get_verse_by_key()` if they don't exist. Keep them private to the repository.

### Step 3: Refactor Node ID Construction (1-2 hours)

Find all places where node IDs are **constructed** (not just parsed):

**Search for:**
```bash
cd rust
rg 'format!\(".*:\' crates/iqrah-storage/src/
```

**Replace patterns:**
```rust
// Old:
let node_id = format!("{}:{}", chapter, verse);

// New:
let node_id = node_id::verse(chapter, verse);
```

**Likely locations:**
- Methods returning node IDs from queries
- Test code constructing sample IDs

### Step 4: Update Error Handling (1 hour)

**File:** `rust/crates/iqrah-storage/src/error.rs`

Add error variant for invalid node IDs:
```rust
#[error("Invalid node ID: {node_id} - {reason}")]
InvalidNodeId {
    node_id: String,
    reason: String,
},
```

Convert `NodeIdError` to `StorageError`:
```rust
impl From<iqrah_core::domain::error::NodeIdError> for StorageError {
    fn from(err: iqrah_core::domain::error::NodeIdError) -> Self {
        StorageError::InvalidNodeId {
            node_id: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}
```

### Step 5: Update Tests (1-2 hours)

**File:** `rust/crates/iqrah-storage/tests/content_repository_test.rs` (or similar)

Update tests to use node_id builders:
```rust
use iqrah_core::domain::node_id;

#[tokio::test]
async fn test_get_verse_node() {
    let repo = setup_test_repo().await;

    // Old: let id = "1:1";
    let id = node_id::verse(1, 1);

    let node = repo.get_node(&id).await.unwrap();
    assert!(node.is_some());
}
```

Find all test cases using hardcoded node IDs and replace with builders.

### Step 6: Run Full Test Suite (30 min)

```bash
cd rust
cargo test --package iqrah-storage
cargo test --package iqrah-core
```

Fix any failing tests. Common issues:
- Node ID format mismatches
- Missing imports
- Error type conversions

## Verification Plan

### Unit Tests

```bash
cd rust
cargo test --package iqrah-storage content_repository
```

- [ ] `test_get_verse_node()` passes
- [ ] `test_get_chapter_node()` passes
- [ ] `test_get_word_node()` passes
- [ ] `test_get_knowledge_node()` passes (if exists)
- [ ] Error handling tests pass (malformed IDs return errors)

### Integration Tests

```bash
cd rust
cargo test --package iqrah-storage --test '*'
```

- [ ] All integration tests pass
- [ ] No panics on malformed node IDs

### CLI Tests

```bash
cd rust
cargo run --bin iqrah-cli -- schedule --goal memorization:chapters-1-3
```

- [ ] Scheduler works (uses repository to get nodes)
- [ ] Sessions generated successfully
- [ ] No errors in logs

### Regression Check

Run the full test suite to ensure no regressions:
```bash
cd rust
RUSTFLAGS="-D warnings" cargo test --all-features
```

- [ ] All tests pass (not just storage tests)
- [ ] No new warnings introduced

## Scope Limits & Safeguards

### ✅ MUST DO

- Replace all string parsing in `get_node()` with node_id module
- Update all node ID construction to use builders
- Add error handling for invalid node IDs
- Update tests to use node_id builders
- Remove integer guessing logic (if num < 114)

### ❌ DO NOT

- Change database schema or queries
- Modify node ID formats (use existing formats from Task 1.3)
- Refactor unrelated code (stay focused on node ID handling)
- Change public API of ContentRepository (only internal implementation)
- Touch Flutter/UI code

### ⚠️ If Uncertain

- If a method's parsing logic is complex → break it into helper methods first
- If tests fail after refactoring → check node ID format matches migration data (must use prefixed format)
- If you find node IDs in other repositories (user repository) → note them but don't change (out of scope)
- If knowledge node handling is unclear → they ARE stored in `node_metadata` table with their full ID (e.g., `VERSE:1:1:memorization`)

## Success Criteria

- [ ] `get_node()` uses `node_id::node_type()` for type detection
- [ ] All parsing logic uses `node_id::parse_*()` functions
- [ ] All node ID construction uses `node_id::*()` builders
- [ ] No more ad-hoc `split(':')` in repository code
- [ ] No more integer guessing (if num < 114)
- [ ] All storage tests pass
- [ ] Integration tests pass
- [ ] CLI test (`iqrah schedule`) works
- [ ] CI checks pass (build, clippy, test, fmt)
- [ ] No new warnings or errors

## Related Files

**Modify These Files:**
- `/rust/crates/iqrah-storage/src/content/repository.rs` - Main refactoring
- `/rust/crates/iqrah-storage/src/error.rs` - Add InvalidNodeId error
- `/rust/crates/iqrah-storage/tests/content_repository_test.rs` - Update tests
- `/rust/crates/iqrah-storage/Cargo.toml` - Ensure iqrah-core dependency

**Reference These Files:**
- `/rust/crates/iqrah-core/src/domain/node_id.rs` - Utility functions (from Task 1.3)

**Impacts These Components:**
- Session service (uses get_node)
- Learning service (uses get_node)
- Exercise service (uses content repository)

## Notes

### Node ID Format Standard

**IMPORTANT:** Only prefixed format is supported:
- `"VERSE:1:1"` ✅
- `"1:1"` ❌ (rejected with error)

No backward compatibility for unprefixed IDs. All migration data MUST use prefixed format.

### Knowledge Nodes

Knowledge nodes like `"VERSE:1:1:memorization"` **ARE stored in the database** in the `node_metadata` table with their full ID as the primary key. They are NOT virtual nodes - they are real database entities with:
- `node_id`: e.g., `"VERSE:1:1:memorization"`
- Scores: `foundational_score`, `influence_score`, etc.
- Graph edges connecting them

Query them directly: `SELECT * FROM node_metadata WHERE node_id = 'VERSE:1:1:memorization'`

### Performance

This refactoring should have **no performance impact**. String parsing is just as fast, but now it's centralized and type-safe.

### Future Work

After this task, we have a clean abstraction for node IDs. Future improvements could include:
- Caching parsed node IDs
- Batch node fetching with type safety
- Compile-time node ID validation (advanced)

But for now, the focus is on eliminating brittle string parsing.
