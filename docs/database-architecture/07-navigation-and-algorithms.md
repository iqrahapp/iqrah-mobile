# Navigation and Graph Traversal Algorithms

**Related Question:** Q7 - Do algorithms infer node IDs or use graph edges? Is logic exposed or abstracted?

## Overview

The Iqrah codebase uses **TWO different strategies** for graph navigation:

1. **ID Inference** - For structural navigation (prev/next word)
2. **Edge Traversal** - For semantic relationships (energy propagation)

This is a **deliberate design choice**, not an oversight.

## Q7: ID Inference vs. Edge Traversal

### Strategy 1: ID Inference (Structural Navigation)

**Use Case:** Get adjacent words (previous/next in sequence).

**Location:** [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 256-314)

**Implementation:**
```rust
async fn get_adjacent_words(&self, word_node_id: &str)
    -> Result<(Option<Node>, Option<Node>)>
{
    // Parse ID: "WORD:1:1:3" → chapter=1, verse=1, position=3
    let parts: Vec<&str> = word_node_id.split(':').collect();

    if parts.len() != 4 || parts[0] != "WORD" {
        return Err(Error::InvalidNodeId(word_node_id.to_string()));
    }

    let chapter: i32 = parts[1].parse()?;
    let verse: i32 = parts[2].parse()?;
    let position: i32 = parts[3].parse()?;

    // --- Get Previous Word ---

    // Try same verse, position - 1
    let prev_word_id = format!("WORD:{}:{}:{}", chapter, verse, position - 1);
    let mut prev_word = self.get_node(&prev_word_id).await?;

    // If not found and not first word in verse, get last word of previous verse
    if prev_word.is_none() && position == 1 && verse > 1 {
        let pattern = format!("WORD:{}:{}:%", chapter, verse - 1);
        prev_word = sqlx::query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes
             WHERE id LIKE ?
             ORDER BY id DESC
             LIMIT 1"
        )
        .bind(pattern)
        .fetch_optional(&self.pool)
        .await?
        .map(|r| r.into());
    }

    // --- Get Next Word ---

    // Try same verse, position + 1
    let next_word_id = format!("WORD:{}:{}:{}", chapter, verse, position + 1);
    let mut next_word = self.get_node(&next_word_id).await?;

    // If not found, try first word of next verse
    if next_word.is_none() {
        let pattern = format!("WORD:{}:{}:1", chapter, verse + 1);
        next_word = self.get_node(&pattern).await?;
    }

    Ok((prev_word, next_word))
}
```

**How it Works:**

1. **Parse the ID** to extract chapter, verse, position
2. **Compute previous ID** by decrementing position: `WORD:1:1:3` → `WORD:1:1:2`
3. **Compute next ID** by incrementing position: `WORD:1:1:3` → `WORD:1:1:4`
4. **Fallback to SQL LIKE queries** when crossing verse boundaries

**Cognitive Load:**
- ✅ **Repository handles it** - parsing logic is encapsulated
- ❌ **At call site** - just call `get_adjacent_words(id)`, get `(prev, next)` back
- **No exposed complexity**

**Performance:**
- **Best case:** O(1) - direct ID lookup (single SELECT by primary key)
- **Crossing boundaries:** O(log n) - LIKE query with ORDER BY and LIMIT

**Example:**
```rust
// Call site
let (prev, next) = content_repo.get_adjacent_words("WORD:1:1:3").await?;

// Returns:
// prev = Some(Node { id: "WORD:1:1:2", ... })
// next = Some(Node { id: "WORD:1:1:4", ... })
```

### Strategy 2: Edge Traversal (Semantic Relationships)

**Use Case:** Energy propagation through knowledge graph.

**Location:** [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs)

**Implementation:**
```rust
async fn propagate_energy(
    &self,
    user_id: &str,
    source_node_id: &str,
    source_energy: f64,
) -> Result<()> {
    // Get all outgoing edges from source node
    let edges = self.content_repo.get_edges_from(source_node_id).await?;

    for edge in edges {
        // Calculate energy transfer based on edge distribution
        let energy_change = self.calculate_energy_transfer(
            source_energy,
            edge.distribution_type,
            edge.param1,
            edge.param2,
        );

        // Update target node energy
        self.user_repo.update_energy(&edge.target_id, energy_change).await?;

        // Log propagation
        self.user_repo.log_propagation(PropagationEvent {
            source_node_id: source_node_id.to_string(),
            target_node_id: edge.target_id.clone(),
            energy_change,
            reason: format!("{:?} edge", edge.edge_type),
        }).await?;
    }

    Ok(())
}
```

**How it Works:**

1. **Query edges table:** `SELECT * FROM edges WHERE source_id = ?`
2. **For each edge:** Compute energy transfer based on distribution
3. **Update target:** Apply energy change to target node

**No ID inference** - purely uses graph structure.

**Performance:**
- **Query edges:** O(E) where E = number of outgoing edges
- **Typical:** E = 5-20 edges per node (word → verse, word → adjacent words, etc.)

**Example:**
```
Source: WORD_INSTANCE:1:1:1:memorization (energy = 0.8)

Edges:
  → VERSE:1:1:memorization (dependency edge, normal dist)
  → WORD_INSTANCE:1:1:1:translation (knowledge edge, const dist)
  → WORD_INSTANCE:1:1:2:contextual_memorization (knowledge edge, normal dist)

Result:
  VERSE:1:1:memorization energy += 0.15
  WORD_INSTANCE:1:1:1:translation energy += 0.32
  WORD_INSTANCE:1:1:2:contextual_memorization energy += 0.11
```

## Why Two Strategies?

### ID Inference for Structure

**Reasons:**

1. **Performance:** Direct lookup is O(1), edge traversal is O(E)
2. **Simplicity:** Word sequence is always linear (word 1, word 2, word 3, ...)
3. **Predictability:** Next word ID is deterministic
4. **Compact storage:** No need to store "next word" edges for every word
5. **Semantic clarity:** String IDs encode position (easier debugging)

**When to Use:**
- Sequential navigation (prev/next word, prev/next verse)
- Hierarchical lookups (verse → chapter)
- Anything where relationship is 1:1 and predictable

### Edge Traversal for Semantics

**Reasons:**

1. **Flexibility:** Knowledge relationships are complex (many-to-many)
2. **Probabilistic:** Energy transfer varies based on distribution params
3. **Extensibility:** Easy to add new edge types without changing code
4. **Graph theory:** Leverages graph structure for learning relationships
5. **Audit trail:** Can log which edges caused energy changes

**When to Use:**
- Energy propagation
- Knowledge synergies (translation → memorization)
- Contextual relationships (word sequence context)
- Anything where relationship is many-to-many or weighted

## Abstraction Quality Analysis

### Repository Abstraction

**Interface (Trait):**
```rust
// ports/content_repository.rs
#[async_trait]
pub trait ContentRepository {
    async fn get_adjacent_words(&self, word_node_id: &str)
        -> Result<(Option<Node>, Option<Node>)>;

    async fn get_edges_from(&self, source_id: &str) -> Result<Vec<Edge>>;
}
```

**Implementation Details Hidden:**
- Call site doesn't know about ID parsing
- Call site doesn't know about LIKE queries
- Call site doesn't know about edge table structure

**Call Site Example:**
```rust
// services/some_service.rs

// Clean API - no exposed implementation details
let (prev, next) = content_repo.get_adjacent_words(word_id).await?;

// vs. what would be needed without abstraction:
// let parts: Vec<&str> = word_id.split(':').collect();
// let chapter = parts[1].parse::<i32>()?;
// let verse = parts[2].parse::<i32>()?;
// let position = parts[3].parse::<i32>()?;
// let prev_id = format!("WORD:{}:{}:{}", chapter, verse, position - 1);
// let prev = content_repo.get_node(&prev_id).await?;
// ... (lots of boilerplate)
```

**Verdict:** ✅ **Excellent abstraction** - cognitive load is properly encapsulated.

### Service Layer Abstraction

**Learning Service API:**
```rust
pub async fn process_review(
    &self,
    user_id: &str,
    node_id: &str,
    grade: ReviewGrade,
) -> Result<ReviewResult>
```

**Caller doesn't need to know:**
- How FSRS parameters are calculated
- How energy propagation works
- Which edges are traversed
- How distributions are sampled

**Verdict:** ✅ **Clean separation** - business logic encapsulated in services.

## Graph Navigation Patterns

### Pattern 1: Sequential Word Iteration

**Use Case:** Display all words in a verse.

**Implementation:**
```rust
async fn get_words_in_ayahs(&self, verse_keys: &[String]) -> Result<Vec<Node>> {
    let patterns: Vec<String> = verse_keys
        .iter()
        .map(|key| format!("WORD_INSTANCE:{}:%", key))
        .collect();

    let mut nodes = Vec::new();
    for pattern in patterns {
        let rows = sqlx::query_as::<_, NodeRow>(
            "SELECT id, node_type, created_at FROM nodes
             WHERE id LIKE ?
             ORDER BY id ASC"
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;

        nodes.extend(rows.into_iter().map(|r| r.into()));
    }

    Ok(nodes)
}
```

**Strategy:** LIKE query with pattern matching.

**Example:**
```sql
SELECT * FROM nodes WHERE id LIKE 'WORD_INSTANCE:1:1:%' ORDER BY id ASC
```

Returns all words in verse 1:1 in order.

**Performance:** O(n log n) where n = words in verse (~10-20 typically).

### Pattern 2: Hierarchical Lookup

**Use Case:** Get verse from word, get chapter from verse.

**Implementation:** Parse ID, construct parent ID.

```rust
fn get_verse_from_word(word_id: &str) -> String {
    // "WORD_INSTANCE:1:1:3" → "VERSE:1:1"
    let parts: Vec<&str> = word_id.split(':').collect();
    format!("VERSE:{}:{}", parts[1], parts[2])
}

fn get_chapter_from_verse(verse_id: &str) -> String {
    // "VERSE:1:1" → "CHAPTER:1"
    let parts: Vec<&str> = verse_id.split(':').collect();
    format!("CHAPTER:{}", parts[1])
}
```

**Strategy:** String manipulation (no DB query needed!).

**Performance:** O(1).

### Pattern 3: Graph Traversal (BFS/DFS)

**Current Status:** NOT IMPLEMENTED.

**Potential Use Case:** "Get all words dependent on this lemma."

**How it Would Work:**
```rust
async fn get_all_dependents(&self, root_id: &str) -> Result<Vec<Node>> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(root_id.to_string());

    while let Some(node_id) = queue.pop_front() {
        if visited.contains(&node_id) {
            continue;
        }
        visited.insert(node_id.clone());

        let edges = self.get_edges_to(&node_id).await?;  // Get incoming edges
        for edge in edges {
            queue.push_back(edge.source_id);
        }
    }

    // Convert visited IDs to nodes
    self.get_nodes(&visited.into_iter().collect::<Vec<_>>()).await
}
```

**Not needed currently** - no feature requires full graph traversal.

## Trade-offs Analysis

### ID Inference

**Pros:**
- ✅ Fast (O(1) for common case)
- ✅ Compact (no edges stored for sequential relationships)
- ✅ Debuggable (IDs are human-readable)
- ✅ Simple to implement

**Cons:**
- ❌ Brittle (assumes ID format never changes)
- ❌ Coupling (repository must know ID structure)
- ❌ Limited flexibility (can't change word ordering)

**Mitigation:**
- ID format is well-defined in Python (identifiers.py)
- Node ID structure is stable (unlikely to change)
- Parsing logic is centralized in repository (single point of change)

### Edge Traversal

**Pros:**
- ✅ Flexible (relationships can change without code changes)
- ✅ Extensible (add new edge types easily)
- ✅ Graph-theoretic (leverages graph algorithms)
- ✅ Audit trail (can log edge usage)

**Cons:**
- ❌ Slower (requires DB query per traversal)
- ❌ Storage overhead (edges table can be large)
- ❌ More complex (need to understand edge semantics)

**Mitigation:**
- Use for semantic relationships only (not structural)
- Index edges table (fast lookups)
- Batch edge queries when possible

## Comparison Table

| Aspect | ID Inference | Edge Traversal |
|--------|-------------|----------------|
| **Use case** | Sequential, hierarchical | Semantic, knowledge |
| **Performance** | O(1) - O(log n) | O(E) where E = edge count |
| **Storage** | Minimal (IDs only) | Edges table (can be large) |
| **Flexibility** | Low (hardcoded structure) | High (data-driven) |
| **Complexity** | Low (string operations) | Medium (graph queries) |
| **Abstraction** | ✅ Well hidden | ✅ Well hidden |
| **Debugging** | ✅ Easy (readable IDs) | ⚠️ Harder (opaque edges) |
| **Example** | prev/next word | energy propagation |

## Graph Design Exploitation

**Q7 Part 2:** Does current usage show understanding of graph design?

### Evidence of Graph Understanding

✅ **Directed Edges:** Code correctly uses `source_id` → `target_id` directionality.

✅ **Edge Types:** Distinguishes Dependency (0) vs Knowledge (1) edges.

✅ **Distributions:** Uses distribution parameters for probabilistic energy transfer.

✅ **Hierarchical Structure:** Implicitly understood via edge creation (word → verse → chapter).

### Evidence of Limited Understanding

❌ **Word Sequences:** Uses ID inference instead of dependency edges for navigation.

❌ **Axis Semantics:** Doesn't parse or understand knowledge axis nodes (see [06-knowledge-axis-design.md](06-knowledge-axis-design.md)).

❌ **Graph Algorithms:** No BFS/DFS traversal (not needed yet, but could be useful).

❌ **Edge Semantics:** Doesn't distinguish cross-axis edges from hierarchical edges in logic.

### Overall Assessment

**Partial Understanding:**
- ✅ Graph structure is imported and used correctly for energy propagation
- ✅ Edges are queried and traversed appropriately
- ⚠️ Structural navigation avoids graph edges (by design, for performance)
- ❌ Semantic meaning of edges not fully exploited (axis-agnostic)

**This is acceptable** - the system uses the right tool for each job (ID inference for structure, edges for semantics).

## Recommendations

### Option 1: Keep Hybrid Approach (Recommended)

**Rationale:**
- ID inference is faster for sequential navigation
- Edge traversal is better for semantic relationships
- Both are well-abstracted behind repository interface

**No changes needed** - current design is solid.

### Option 2: Unify on Edge Traversal

**Add structural edges:**
```sql
-- For every word, add "next word" edge
INSERT INTO edges (source_id, target_id, edge_type) VALUES
  ('WORD:1:1:1', 'WORD:1:1:2', 0),
  ('WORD:1:1:2', 'WORD:1:1:3', 0),
  ...
```

**Change navigation:**
```rust
async fn get_next_word(&self, word_id: &str) -> Result<Option<Node>> {
    let edges = self.get_edges_from(word_id).await?;
    let next_edge = edges.iter().find(|e| e.edge_type == EdgeType::Dependency)?;
    self.get_node(&next_edge.target_id).await
}
```

**Pros:** Pure graph approach, no ID inference.

**Cons:** Slower, more storage, no real benefit.

**Verdict:** NOT RECOMMENDED - hybrid approach is superior.

### Option 3: Add Graph Utilities

**Helper functions for common patterns:**
```rust
pub mod graph_utils {
    pub fn parse_word_id(id: &str) -> Result<(i32, i32, i32)> {
        // Returns (chapter, verse, position)
    }

    pub fn construct_word_id(chapter: i32, verse: i32, position: i32) -> String {
        format!("WORD:{}:{}:{}", chapter, verse, position)
    }

    pub fn get_parent_id(node_id: &str, node_type: NodeType) -> Option<String> {
        // WORD_INSTANCE → WORD, VERSE → CHAPTER, etc.
    }
}
```

**Benefit:** Centralize ID manipulation logic, make it reusable.

## File Locations

**ID Inference Implementation:**
- [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 256-314) - `get_adjacent_words`
- [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 161-195) - `get_words_in_ayahs`

**Edge Traversal Implementation:**
- [content/repository.rs](../../rust/crates/iqrah-storage/src/content/repository.rs) (lines 81-95) - `get_edges_from`
- [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs) - Energy propagation

**Python ID Generation:**
- [graph/identifiers.py](../../research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/identifiers.py) - Canonical ID format definitions

---

**Navigation:** [← Knowledge Axis Design](06-knowledge-axis-design.md) | [Next: Flexible Content Import →](08-flexible-content-import.md)
