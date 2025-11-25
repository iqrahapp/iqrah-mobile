# Knowledge Node Redesign: Documentation Index

**Date**: 2025-01-25
**Status**: Ready for Implementation
**Decision**: Physical Nodes with Integer-based Graph (Option 4 Enhanced)

---

## Quick Links

### Essential Reading (Start Here)
1. [Architecture Decision](architecture/00-knowledge-node-redesign-decision.md) - **START HERE**
2. [Design Drift Analysis](architecture/01-design-drift-analysis.md) - Background context
3. [Task 0: Update Production-Ready Docs](agent-tasks/task-0-update-production-ready-docs.md) - **FIRST TASK**

### Implementation Guides
- [Schema Design](implementation/schema-design.md) - Complete DDL and examples
- [Rust Implementation Guide](implementation/rust-implementation-guide.md) - NodeRegistry, Repository refactor
- [Python Generator Guide](implementation/python-generator-guide.md) - Two-phase generation

### Reference Materials
- [Enum Mappings](reference/enum-mappings.md) - INTEGER values (CRITICAL)
- [Validation Checklist](reference/validation-checklist.md) - Post-implementation checks

---

## Overview

### What Changed?

**BEFORE**: Virtual knowledge nodes constructed on-demand
**AFTER**: Physical nodes with integer-based graph operations

### Core Principle

**"Internal Ints, External Strings"**
- Graph operations use INTEGER IDs (performance: O(1) lookups)
- User data uses STRING keys (stability across content updates)
- NodeRegistry maps between the two

---

## Documentation Structure

```
docs/
├── architecture/
│   ├── 00-knowledge-node-redesign-decision.md    ⭐ Architectural decision
│   └── 01-design-drift-analysis.md               Historical analysis
│
├── implementation/
│   ├── schema-design.md                           Complete DDL & SQL
│   ├── rust-implementation-guide.md               Rust refactor guide
│   └── python-generator-guide.md                  Python two-phase guide
│
├── agent-tasks/
│   └── task-0-update-production-ready-docs.md    ⭐ FIRST TASK (blocking)
│
├── reference/
│   ├── enum-mappings.md                          ⭐ INTEGER mappings (CRITICAL)
│   └── validation-checklist.md                    Post-implementation checks
│
└── knowledge-node-redesign-index.md              This file
```

---

## Implementation Phases

### Phase 0: Documentation (BLOCKING)
**Task**: [task-0-update-production-ready-docs.md](agent-tasks/task-0-update-production-ready-docs.md)
- Update all 15 production-ready tasks for integer architecture
- Remove virtual node references
- Timeline: 4-6 hours
- **Must complete before Phase 1**

### Phase 1: Schema & Rust Core
**Agent Tasks**:
- Task A: Schema design & migration SQL
- Task B1: Rust NodeRegistry implementation
- Task B3: Repository layer refactor

**Timeline**: 1-2 days with parallel execution

### Phase 2: Python Generator
**Agent Task**:
- Task B2: Python two-phase generation

**Timeline**: 1 day (parallel with Phase 1)

### Phase 3: Integration & Testing
**Agent Task**:
- Task C: Integration testing

**Timeline**: 0.5 days

### Phase 4: Documentation Updates
**Agent Task**:
- Task D: Update documentation based on implementation

**Timeline**: 0.5 days

### Phase 5: Final Validation
**Agent Task**:
- Task E: QA, performance benchmarks, CI checks

**Timeline**: 0.5 days

**Total Timeline**: 1-2 days with 7-8 agents in parallel

---

## Key Design Elements

### 1. Node Registry Pattern

```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,        -- Internal RowID for performance
    ukey TEXT NOT NULL UNIQUE,     -- Stable string key
    node_type INTEGER NOT NULL     -- Enum value
) STRICT;
```

### 2. Knowledge Node Linking

```sql
CREATE TABLE knowledge_nodes (
    node_id INTEGER PRIMARY KEY,   -- FK to nodes.id
    base_node_id INTEGER NOT NULL, -- FK to nodes.id
    axis INTEGER NOT NULL,         -- Enum: 0-5
    FOREIGN KEY (node_id) REFERENCES nodes(id),
    FOREIGN KEY (base_node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;
```

### 3. Integer-based Edges

```sql
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

### 4. NodeRegistry (Boundary Layer)

```rust
pub struct NodeRegistry {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    pool: SqlitePool,
}

impl NodeRegistry {
    pub async fn get_id(&self, ukey: &str) -> Result<i64>;
    pub async fn get_ukey(&self, id: i64) -> Result<String>;
}
```

---

## Critical Rules

### Enum Mappings (NEVER CHANGE)

See [enum-mappings.md](reference/enum-mappings.md) for complete reference.

**NodeType**:
- 0 = Verse
- 1 = Chapter
- 2 = Word
- 3 = Knowledge
- 4 = WordInstance

**KnowledgeAxis**:
- 0 = Memorization (verse)
- 1 = Translation (verse)
- 2 = Tafsir (verse)
- 3 = Tajweed (verse)
- 4 = ContextualMemorization (word)
- 5 = Meaning (word)

**EdgeType**:
- 0 = Dependency
- 1 = Knowledge

### Referential Integrity

**ALL implementations MUST**:
- Use INTEGER FKs for graph relationships
- Register nodes before creating edges
- Validate FK constraints on import
- Use STRING keys only in user.db

---

## Benefits Summary

### Performance
- **10-100x faster** graph traversal (integer adjacency lists)
- **O(1)** node lookups with caching
- **50-70% less memory** per node ID (8 bytes vs 20-50 bytes)

### Correctness
- **Database-level referential integrity** (no orphan edges)
- **Validation at schema level** (foreign key constraints)
- **No virtual node bugs** (explicit storage)

### Maintainability
- **Clear separation**: content.db vs user.db
- **Easy debugging**: can list/count nodes
- **Future-proof**: extensible node types

### User Data Stability
- **User progress survives content updates**
- **No migration on monthly refreshes**
- **String keys immune to integer ID changes**

---

## FAQ

### Q: Why integer IDs if they change on content updates?
**A**: Graph operations are 10-100x faster with integers. User data uses stable string keys, so integer ID changes in content.db don't affect user.db.

### Q: Can I add a new NodeType?
**A**: Yes, but NEVER reuse existing values. Always append new values (e.g., NewType = 5). Update enum in Rust AND Python.

### Q: How do I look up a node?
**A**: External API uses strings: `repo.get_node("VERSE:1:1")`. Internal API uses integers: `repo.get_node_by_id(123)`.

### Q: What if string key changes?
**A**: String keys (ukeys) are part of the architecture contract and should NEVER change. Adding a new node type or renaming requires migration.

---

## Next Steps

### For Project Lead:
1. Review [Architecture Decision](architecture/00-knowledge-node-redesign-decision.md)
2. Distribute [Task 0](agent-tasks/task-0-update-production-ready-docs.md) to documentation agent
3. Wait for Task 0 completion before distributing implementation tasks

### For AI Agents:
1. **Task 0 Agent**: Start with [task-0-update-production-ready-docs.md](agent-tasks/task-0-update-production-ready-docs.md)
2. **Implementation Agents**: Wait for Task 0 completion, then follow wave-based execution plan

### For Developers:
1. Read [Architecture Decision](architecture/00-knowledge-node-redesign-decision.md)
2. Study [Schema Design](implementation/schema-design.md)
3. Review [Enum Mappings](reference/enum-mappings.md) (CRITICAL)

---

## Status Tracking

### Phase 0: Documentation
- [ ] Task 0: Update production-ready tasks

### Phase 1-2: Core Implementation
- [ ] Task A: Schema design
- [ ] Task B1: NodeRegistry
- [ ] Task B2: Python generator
- [ ] Task B3: Repository refactor

### Phase 3: Integration
- [ ] Task C: Integration testing

### Phase 4: Documentation
- [ ] Task D: Documentation updates

### Phase 5: Validation
- [ ] Task E: Final QA & CI

---

## Contact

**Questions about this redesign?**
- Architecture decisions: See [00-knowledge-node-redesign-decision.md](architecture/00-knowledge-node-redesign-decision.md)
- Implementation details: See respective guides in [implementation/](implementation/)
- Validation: See [validation-checklist.md](reference/validation-checklist.md)

---

**Last Updated**: 2025-01-25
**Version**: 1.0
**Status**: Ready for Implementation
