# ⚠️ ARCHITECTURAL REDESIGN IN PROGRESS

**Date**: 2025-01-25
**Status**: BLOCKING - All tasks require updates before execution

---

## Critical Notice

**DO NOT execute any production-ready tasks until Task 0 is complete.**

A major architectural change has been approved: transitioning from **virtual knowledge nodes** to **physical nodes with integer-based graph** architecture.

---

## What Happened?

During verification of these production-ready tasks, a fundamental architectural inconsistency was discovered:
- **Documentation** described knowledge nodes as physical database rows
- **Implementation** used virtual wrappers constructed on-demand

The project architect has decided to implement **Physical Nodes with Integer-based Graph** (Option 4 Enhanced) for production-grade architecture.

---

## Impact on Production-Ready Tasks

**The following tasks contain outdated information based on virtual nodes:**

### High Priority (Must Update):
- ✅ task-1.1-architecture-documentation.md
- ❌ task-1.4-repository-refactoring.md (**MAJOR REWRITE NEEDED**)
- ❌ task-2.1-generate-full-knowledge-graph.md (**TWO-PHASE GENERATION**)
- ❌ task-2.2-verify-knowledge-axis-end-to-end.md
- ❌ task-2.4-cross-axis-propagation-verification.md
- ❌ task-3.3-graph-update-mechanism.md

### Should Review:
- ✅ AGENT_PROMPT_TEMPLATE.md

**Legend**:
- ❌ = Requires updates
- ✅ = Minor updates or reviewed

---

## Required Action: Complete Task 0 First

**Before distributing ANY tasks to AI agents**, complete:

### Task 0: Update Production-Ready Tasks
**Location**: `/docs/agent-tasks/task-0-update-production-ready-docs.md`

**Purpose**: Update all production-ready tasks for consistency with integer-based architecture

**Timeline**: 4-6 hours
**Priority**: P0 (BLOCKING)

**Deliverables**:
- All 7 task documents updated with integer-based architecture
- Zero "virtual node" references (except historical context)
- All SQL examples using INTEGER IDs
- All Rust examples using NodeRegistry
- Python two-phase generation documented

---

## New Architecture Summary

### Key Principle: "Internal Ints, External Strings"

**content.db** (immutable graph):
```sql
-- Node registry (source of truth)
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,        -- Internal RowID for O(1) lookups
    ukey TEXT NOT NULL UNIQUE,     -- Stable string key
    node_type INTEGER NOT NULL
) STRICT;

-- Integer-based edges
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

**user.db** (mutable user state):
```sql
-- String keys for stability
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_ukey TEXT NOT NULL,       -- Stable across content updates
    -- ... state fields ...
    PRIMARY KEY (user_id, node_ukey)
) STRICT, WITHOUT ROWID;
```

---

## Complete Documentation

**Full documentation available at**:
- **Index**: `/docs/knowledge-node-redesign-index.md`
- **Architecture**: `/docs/architecture/`
- **Implementation**: `/docs/implementation/`
- **Reference**: `/docs/reference/`
- **Agent Tasks**: `/docs/agent-tasks/`

### Essential Reading:
1. [Architecture Decision](/docs/architecture/00-knowledge-node-redesign-decision.md)
2. [Design Drift Analysis](/docs/architecture/01-design-drift-analysis.md)
3. [Schema Design](/docs/implementation/schema-design.md)
4. [Enum Mappings](/docs/reference/enum-mappings.md) ⭐ CRITICAL

---

## Workflow for AI Agents

### Step 1: Documentation Agent (Task 0)
1. Read all reference documentation
2. Update all 7 production-ready task files
3. Verify consistency and correctness
4. Commit changes

### Step 2: Implementation Agents (Tasks A-E)
**Only after Task 0 is complete**:
- Wave 1: Schema design (blocking)
- Wave 2: Rust + Python implementation (parallel)
- Wave 3: Integration testing
- Wave 4: Documentation updates
- Wave 5: Final validation

**Total Timeline**: 1-2 days with 7-8 agents in parallel

---

## Benefits of New Architecture

### Performance
- **10-100x faster** graph traversal
- **O(1)** node lookups with caching
- **50-70% less memory** per node ID

### Correctness
- **Database-level referential integrity**
- **No orphan edges** (foreign key constraints)
- **Explicit node storage** (no virtual bugs)

### Maintainability
- **Clear separation**: content.db vs user.db
- **Easy debugging**: can list/count nodes
- **Future-proof**: extensible architecture

### User Data Stability
- **User progress survives content updates**
- **No migration needed** on monthly refreshes
- **String keys immune** to integer ID changes

---

## Questions?

**For full context**, read:
- [Knowledge Node Redesign Index](/docs/knowledge-node-redesign-index.md)
- [Task 0: Update Production-Ready Docs](/docs/agent-tasks/task-0-update-production-ready-docs.md)

**For architecture decisions**, read:
- [Architecture Decision](/docs/architecture/00-knowledge-node-redesign-decision.md)

---

**Status**: Task 0 must complete before ANY other work proceeds.

**Last Updated**: 2025-01-25
