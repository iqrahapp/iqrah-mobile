# Task 0: Update Production-Ready Tasks for Integer-Based Architecture

**Priority**: P0 (BLOCKING - Must complete before all other tasks)
**Estimated Time**: 4-6 hours
**Agent Type**: Documentation specialist
**Parallelizable**: No (blocks Phase 1 task distribution)

---

## Context

A major architectural change has been approved: transitioning from **virtual knowledge nodes** to **physical nodes with integer-based graph** (Option 4 Enhanced).

The current production-ready tasks (`docs/todo/production-ready-tasks/task-*.md`) contain incorrect assumptions based on the old virtual node implementation. These tasks must be updated for FULL consistency and compatibility with the new architecture BEFORE distributing any implementation work to agents.

---

## Objectives

Update all production-ready task documents to:
1. Replace virtual node references with physical node + node registry
2. Update all SQL examples to use INTEGER IDs
3. Update all Rust examples to use NodeRegistry
4. Ensure referential integrity is mentioned where relevant
5. Add two-phase generation approach to Python tasks
6. Maintain FULL internal consistency across all 15 tasks

---

## Architectural Changes Summary

### What Changed:

**BEFORE (Virtual Nodes)**:
- Knowledge nodes were virtual wrappers constructed on-demand
- `get_node("VERSE:1:1:memorization")` → parse ID, fetch verse, wrap with axis
- No dedicated storage for knowledge nodes
- String-based node IDs throughout

**AFTER (Physical Nodes + Integer IDs)**:
- Knowledge nodes are physical database entities in `nodes` + `knowledge_nodes` tables
- All graph operations use INTEGER IDs for performance
- `nodes` table is the single source of truth (node registry)
- NodeRegistry maps between stable string keys and internal integer IDs
- Python generator uses two-phase approach: register nodes, then create edges

### Key Principles:

1. **"Internal Ints, External Strings"**:
   - Internal graph operations: INTEGER IDs
   - External API (user-facing): STRING unique keys
   - Boundary layer (NodeRegistry): maps between them

2. **Two-Database Architecture**:
   - content.db: INTEGER IDs for performance (immutable graph)
   - user.db: STRING keys for stability (mutable user state)

3. **Referential Integrity**:
   - All relationships use foreign key constraints
   - Database enforces correctness at schema level

---

## Reference Documentation

**CRITICAL**: Read these documents BEFORE starting:

1. [Architecture Decision](/docs/architecture/00-knowledge-node-redesign-decision.md)
2. [Design Drift Analysis](/docs/architecture/01-design-drift-analysis.md)
3. [Schema Design](/docs/implementation/schema-design.md)
4. [Rust Implementation Guide](/docs/implementation/rust-implementation-guide.md)
5. [Python Generator Guide](/docs/implementation/python-generator-guide.md)
6. [Enum Mappings](/docs/reference/enum-mappings.md)

---

## Tasks to Update

### High Priority (Must Update):

#### 1. task-1.1-architecture-documentation.md
**Changes Required**:
- Add Node Registry section
- Document the "Internal Ints, External Strings" principle
- Update schema diagrams to show `nodes` table
- Add explanation of two-database architecture
- Document NodeType, KnowledgeAxis, EdgeType enum mappings

**Sections to Add**:
```markdown
## Node Registry Pattern

The `nodes` table serves as the single source of truth for all graph entities:

```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;
```

All graph operations use INTEGER IDs for O(1) performance, while user data uses STRING unique keys for stability across content updates.
```

---

#### 2. task-1.4-repository-refactoring.md
**Changes Required**: **COMPLETE REWRITE**

This task contains the most significant errors. The entire virtual node explanation must be removed and replaced with integer-based repository implementation.

**Old Content (REMOVE)**:
```markdown
Knowledge nodes like "VERSE:1:1:memorization" ARE stored in the database
in the node_metadata table with their full ID as the primary key.
Query them directly: SELECT * FROM node_metadata WHERE node_id = 'VERSE:1:1:memorization'
```

**New Content (ADD)**:
```markdown
## Node Lookup Strategy

### External API: String-based get_node

```rust
async fn get_node(&self, ukey: &str) -> Result<Option<Node>> {
    // 1. Lookup integer ID from string key
    let node_id = self.registry.get_id(ukey).await?;

    // 2. Use fast integer path
    self.get_node_by_id(node_id).await
}
```

### Internal API: Integer-based get_node_by_id

```rust
async fn get_node_by_id(&self, node_id: i64) -> Result<Option<Node>> {
    let row = query!(
        r#"
        SELECT n.id, n.ukey, n.node_type, kn.base_node_id, kn.axis
        FROM nodes n
        LEFT JOIN knowledge_nodes kn ON kn.node_id = n.id
        WHERE n.id = ?
        "#,
        node_id
    ).fetch_optional(&self.pool).await?;

    // Construct Node from row...
}
```

### NodeRegistry

The repository uses `NodeRegistry` to map between stable string keys and internal integer IDs:

```rust
pub struct NodeRegistry {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    pool: SqlitePool,
}
```
```

**Key Points**:
- Remove ALL references to virtual nodes
- Update ALL code examples to use integer IDs
- Document NodeRegistry usage
- Update all SQL queries to use INTEGER joins

---

#### 3. task-2.1-generate-full-knowledge-graph.md
**Changes Required**: **TWO-PHASE GENERATION**

**Add Section**:
```markdown
## Generation Approach: Two-Phase

### Phase 1: Node Registration

```python
# Register all nodes and get integer IDs
node_id_map = {}  # ukey -> id

for verse in verses:
    ukey = f"VERSE:{verse.chapter}:{verse.num}"
    node_id = builder.register_node(ukey, NodeType.VERSE)
    node_id_map[ukey] = node_id

    # Register knowledge nodes
    for axis in [KnowledgeAxis.MEMORIZATION, ...]:
        kn_ukey = f"{ukey}:{axis.name.lower()}"
        kn_node_id = builder.register_node(kn_ukey, NodeType.KNOWLEDGE)
        node_id_map[kn_ukey] = kn_node_id

        # Link to base node
        builder.add_knowledge_node(kn_node_id, node_id, axis)
```

### Phase 2: Edge Creation

```python
# Create edges using integer IDs
for source_ukey, target_ukey in edge_pairs:
    builder.add_edge(
        source_id=node_id_map[source_ukey],
        target_id=node_id_map[target_ukey],
        edge_type=EdgeType.KNOWLEDGE,
        weight=0.8
    )
```
```

**Update SQL Examples**:
```sql
-- OLD (string-based):
INSERT INTO edges (source_id, target_id, ...) VALUES
    ('VERSE:1:1:memorization', 'VERSE:1:2:memorization', ...);

-- NEW (integer-based):
INSERT INTO edges (source_id, target_id, ...) VALUES
    (101, 105, ...);  -- Integer IDs from nodes table
```

---

#### 4. task-2.2-verify-knowledge-axis-end-to-end.md
**Changes Required**:
- Update test assumptions to expect physical nodes
- Update SQL verification queries to use INTEGER IDs
- Add NodeRegistry string→int mapping in test code

**Old Query (REPLACE)**:
```sql
SELECT * FROM node_metadata WHERE node_id LIKE 'VERSE:%:memorization';
```

**New Query (USE)**:
```sql
-- Query physical knowledge nodes
SELECT n.ukey, kn.axis
FROM nodes n
JOIN knowledge_nodes kn ON n.id = kn.node_id
WHERE kn.axis = 0;  -- Memorization
```

---

#### 5. task-2.4-cross-axis-propagation-verification.md
**Changes Required**:
- Update edge queries to use INTEGER IDs
- Add explanation of integer-based graph traversal
- Update test code to use NodeRegistry for lookups

**Add Section**:
```markdown
## Graph Traversal with Integer IDs

Energy propagation operates on integer IDs for performance:

```rust
async fn propagate_energy(&self, source_node_id: i64, energy: f64) -> Result<()> {
    let edges = query!(
        "SELECT target_id, weight FROM edges WHERE source_id = ?",
        source_node_id
    ).fetch_all(&self.pool).await?;

    for edge in edges {
        self.add_energy(edge.target_id, energy * edge.weight).await?;
    }
}
```
```

---

#### 6. task-3.3-graph-update-mechanism.md
**Changes Required**:
- Clarify content.db replacement strategy (integer IDs change)
- Document user.db stability through string keys
- Update examples to show NodeRegistry usage

**Add Section**:
```markdown
## Content Update Strategy

### content.db Replacement

When content.db is regenerated (monthly):
- All integer node IDs may change
- String unique keys (ukeys) remain stable
- Python generator assigns new integer IDs

### user.db Stability

User state persists using STRING keys:

```sql
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_ukey TEXT NOT NULL,  -- Stable across updates!
    -- ... state fields ...
    PRIMARY KEY (user_id, node_ukey)
) STRICT, WITHOUT ROWID;
```

When loading user state:
1. Query user.db for `node_ukey`
2. Use NodeRegistry to resolve current `node_id`
3. Perform graph operations with integer ID
```

---

### Medium Priority (Should Review):

#### 7. AGENT_PROMPT_TEMPLATE.md
**Changes Required**:
- Add node registry section to "Important Notes"
- Document integer vs string ID usage
- Update architecture references

**Add to "Important Notes"**:
```markdown
- **Node IDs:** The architecture uses INTEGER IDs internally for performance:
  - ✅ Internal operations use `i64` node IDs
  - ✅ External API accepts/returns `String` unique keys (ukeys)
  - ✅ NodeRegistry maps between the two
  - Exception: user.db always uses string keys for stability

- **Graph Operations:** Always use integer IDs for performance:
  ```rust
  // ❌ AVOID: String-based edge queries
  query!("SELECT * FROM edges WHERE source_id = ?", "VERSE:1:1:memorization");

  // ✅ PREFER: Integer-based edge queries
  let node_id = registry.get_id("VERSE:1:1:memorization").await?;
  query!("SELECT * FROM edges WHERE source_id = ?", node_id);
  ```
```

---

## Scope Boundaries

### DO:
- [ ] Update all SQL examples to use INTEGER IDs
- [ ] Remove all "virtual node" references
- [ ] Add NodeRegistry mentions where relevant
- [ ] Document two-phase Python generation
- [ ] Update Rust code examples for integer IDs
- [ ] Ensure referential integrity is mentioned
- [ ] Verify internal consistency across all tasks

### DO NOT:
- [ ] Change the task objectives or scope
- [ ] Remove existing test cases (update them instead)
- [ ] Add new features beyond architectural consistency
- [ ] Modify files outside `docs/todo/production-ready-tasks/`

---

## Verification Checklist

After updates, verify:

### 1. Terminology Consistency
```bash
cd docs/todo/production-ready-tasks

# No "virtual node" references (except historical context)
grep -r "virtual" *.md | grep -v "historical" | grep -v "before"
# Expected: 0 matches

# NodeRegistry mentioned in relevant tasks
grep -r "NodeRegistry" *.md
# Expected: Matches in task-1.4, task-2.1, task-2.2

# Integer ID usage in SQL
grep -r "INTEGER" *.md
# Expected: Multiple matches in schema sections
```

### 2. SQL Example Correctness
- [ ] All edge queries use INTEGER IDs
- [ ] All node queries join through `nodes` table
- [ ] Foreign key references use integers

### 3. Rust Example Correctness
- [ ] Repository examples use `NodeRegistry`
- [ ] Graph traversal uses integer IDs
- [ ] Boundary layer converts to strings for user.db

### 4. Python Example Correctness
- [ ] Two-phase generation approach documented
- [ ] Enum values use integer constants
- [ ] Node registration before edge creation

---

## Acceptance Criteria

- [ ] All 7 task documents updated
- [ ] Zero references to "virtual nodes" (except historical)
- [ ] All SQL examples use INTEGER IDs
- [ ] All Rust examples use NodeRegistry
- [ ] Python two-phase generation documented
- [ ] Internal consistency across all tasks verified
- [ ] Terminology alignment with architecture docs
- [ ] Verification checklist passes

---

## Deliverables

1. Updated task files (7 files):
   - task-1.1-architecture-documentation.md
   - task-1.4-repository-refactoring.md
   - task-2.1-generate-full-knowledge-graph.md
   - task-2.2-verify-knowledge-axis-end-to-end.md
   - task-2.4-cross-axis-propagation-verification.md
   - task-3.3-graph-update-mechanism.md
   - AGENT_PROMPT_TEMPLATE.md

2. Verification report:
   - Checklist results
   - List of changes per file
   - Confirmation of internal consistency

---

## Timeline

**Estimated**: 4-6 hours
- Reading reference docs: 1 hour
- Updating task-1.4 (major rewrite): 1.5 hours
- Updating task-2.1 (two-phase generation): 1 hour
- Updating tasks 2.2, 2.4, 3.3: 1 hour
- Updating task-1.1 and template: 0.5 hour
- Verification and consistency check: 1 hour

---

## Success Criteria

Your work is complete when:
- [ ] All 7 files updated with architectural changes
- [ ] Verification checklist passes 100%
- [ ] Internal consistency confirmed across all tasks
- [ ] No "virtual node" references remain
- [ ] SQL/Rust/Python examples updated correctly
- [ ] Git commit created with clear message

---

## Notes for AI Agent

**Read these docs FIRST before making any changes**:
1. `/docs/architecture/00-knowledge-node-redesign-decision.md`
2. `/docs/implementation/schema-design.md`
3. `/docs/reference/enum-mappings.md`

**Key Points**:
- This is documentation-only work (no code changes)
- Focus on consistency and correctness
- Task-1.4 needs the most significant rewrite
- Use examples from reference docs, don't invent new patterns
- Verify changes against enum mappings for correctness

**Questions?** If anything is unclear, ask before proceeding.

---

## References

- [Architecture Decision](/docs/architecture/00-knowledge-node-redesign-decision.md)
- [Schema Design](/docs/implementation/schema-design.md)
- [Rust Guide](/docs/implementation/rust-implementation-guide.md)
- [Python Guide](/docs/implementation/python-generator-guide.md)
- [Enum Mappings](/docs/reference/enum-mappings.md)
