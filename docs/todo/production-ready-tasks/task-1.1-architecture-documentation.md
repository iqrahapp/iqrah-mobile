# Task 1.1: Document Architecture & Node ID Contracts

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None
- **Agent Type:** Research + Documentation
- **Parallelizable:** Yes (with tasks 1.2, 1.3, 1.5)

## Goal

Create authoritative architecture documentation defining the 2-DB design rationale, node ID format contracts, and stability policy to prevent breaking changes.

## Context

The project has evolved from MVP (1 DB, generic nodes) → v2 (2 DBs, purist schema). However, there's no single authoritative document explaining:
- Why 2 DBs instead of 3
- Strict node ID formats and parsing rules
- Node ID stability guarantees (critical for user progress preservation)
- Graph update strategy (monthly erase/replace)

AI agents implementing subsequent tasks need clear contracts to avoid hallucinating inconsistent ID formats.

## Current State

**Existing Documentation:**
- [docs/database-architecture/](/docs/database-architecture/) - Comprehensive audit from Sprint 7
- [docs/content-db-schema.md](/docs/content-db-schema.md) - v2 schema details
- [docs/todo/scheduler-v2-knowledge-graph.md](/docs/todo/scheduler-v2-knowledge-graph.md) - Scheduler design
- Node IDs mentioned in multiple places but no single source of truth

**Current Node ID Usage (Inconsistent):**
- Content nodes: `"VERSE:1:1"`, `"CHAPTER:1"`, `"WORD:123"` (sometimes)
- Knowledge nodes: `"VERSE:1:1:memorization"`, `"WORD_INSTANCE:1:1:3:translation"`
- Repository parsing: Ad-hoc string splitting in multiple places

**Problem:** No documented guarantee that node IDs won't change, risking user progress loss.

## Target State

**New Document:** `docs/architecture/data-architecture-v2.md`

**Contents:**
1. **Database Design Rationale**
   - Why 2 DBs (content.db + user.db) not 3
   - What belongs in each DB
   - Graph update strategy (monthly erase/replace)
   - Package installation strategy (insert into content.db)

2. **Node ID Format Specification**
   - **Content Nodes:**
     - `CHAPTER:{chapter_num}` (e.g., `CHAPTER:1`)
     - `VERSE:{chapter}:{verse}` (e.g., `VERSE:1:1`)
     - `WORD:{word_id}` (e.g., `WORD:123`) - word_id from DB autoincrement
     - `WORD_INSTANCE:{chapter}:{verse}:{position}` (e.g., `WORD_INSTANCE:1:1:3`)

   - **Knowledge Nodes:**
     - `{content_node_id}:{axis}` (e.g., `VERSE:1:1:memorization`)
     - Valid axes: `memorization`, `translation`, `tafsir`, `tajweed`, `contextual_memorization`, `meaning`

   - **Parsing Rules:**
     - Split on `:` delimiter
     - Validate prefix matches enum
     - Validate numeric parts in valid ranges
     - Return typed errors for malformed IDs

3. **Node ID Stability Policy**
   - **Guarantee:** Once a node ID is released in production, it MUST NOT change or be removed
   - **Rationale:** User progress (user_memory_states) is keyed by node_id
   - **Enforcement:** Python build pipeline validation (Task 1.5)
   - **Exceptions:** Only via explicit migration with user data mapping

4. **Graph Update Process**
   - **Frequency:** Monthly (not daily/weekly)
   - **Method:** Erase & replace
     ```sql
     BEGIN TRANSACTION;
     DELETE FROM edges;
     DELETE FROM node_metadata;
     DELETE FROM goals;
     DELETE FROM node_goals;
     -- INSERT new graph data
     COMMIT;
     ```
   - **Preservation:** User progress in user.db is untouched
   - **Validation:** New graph must contain all node IDs from old graph (Task 1.5)

5. **Schema Version Management**
   - Format: `major.minor.patch` (semantic versioning)
   - Breaking changes (node ID format changes) → major bump
   - Graph updates (new nodes/edges) → minor bump
   - Bug fixes (score corrections) → patch bump

## Implementation Steps

### Step 1: Read Existing Documentation (1-2 hours)

Read and synthesize:
- [docs/database-architecture/01-migration-history-from-mvp.md](/docs/database-architecture/01-migration-history-from-mvp.md)
- [docs/database-architecture/02-current-state-assessment.md](/docs/database-architecture/02-current-state-assessment.md)
- [docs/database-architecture/08-v2-implementation-priority-plan.md](/docs/database-architecture/08-v2-implementation-priority-plan.md)
- [docs/content-db-schema.md](/docs/content-db-schema.md)

### Step 2: Analyze Current Node ID Usage (1-2 hours)

Examine these files to understand current patterns:
- `rust/crates/iqrah-core/src/domain/models.rs` (lines 54-147) - KnowledgeAxis enum
- `rust/crates/iqrah-storage/src/content/repository.rs` (lines 32-80) - get_node() parsing logic
- `research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py` - Python node ID generation

Document all observed ID formats.

### Step 3: Create Architecture Document (3-4 hours)

Create `docs/architecture/data-architecture-v2.md` with sections outlined in Target State above.

**Template:**
```markdown
# Data Architecture v2: Production Design

**Version:** 2.0.0
**Last Updated:** 2024-11-24
**Status:** Authoritative

## Executive Summary
[1-2 paragraphs: 2-DB design, node ID contracts, stability guarantees]

## Database Design
### Architecture Decision: 2 Databases
[Rationale, content.db vs user.db responsibilities]

### Graph Update Strategy
[Monthly erase/replace process]

## Node ID Specification
### Content Node Formats
[Detailed format specs with examples]

### Knowledge Node Formats
[Axis-based format specs]

### Parsing and Validation Rules
[How to parse, validate, and handle errors]

## Node ID Stability Policy
### Guarantees
[What we promise users]

### Enforcement
[How we prevent breaking changes]

### Migration Process
[If IDs must change, how to handle]

## Schema Versioning
[Semantic versioning strategy]

## Graph Update Process
[Monthly update procedure]

## Related Documentation
[Links to other relevant docs]
```

### Step 4: Create docs/architecture Directory (if needed)

```bash
mkdir -p docs/architecture
```

### Step 5: Validate with Existing Code (1 hour)

Cross-reference your documented ID formats with:
- All ID formats in `migrations_content/20241118000001_knowledge_graph_chapters_1_3.sql`
- All parsing logic in `rust/crates/iqrah-storage/src/content/repository.rs`
- All ID generation in Python R&D project

Ensure no contradictions.

## Verification Plan

### Documentation Review Checklist

- [ ] **Completeness:** All 5 major sections present (DB design, node IDs, stability, updates, versioning)
- [ ] **Clarity:** Node ID formats have examples for each type
- [ ] **Consistency:** ID formats match current Rust domain models
- [ ] **Actionable:** Stability policy is enforceable (linked to Task 1.5)
- [ ] **Referenced:** Links to related docs (DB architecture audit, schema docs)

### Cross-Reference Validation

- [ ] **Python alignment:** Check `knowledge_builder.py` generates IDs matching documented format
- [ ] **Rust alignment:** Check `models.rs` KnowledgeAxis enum matches documented axes
- [ ] **Data alignment:** Check migration SQL uses documented ID formats

### Manual Review

- [ ] Read the document end-to-end as if you're a new AI agent
- [ ] Can you understand the 2-DB design rationale?
- [ ] Can you parse node IDs following the rules?
- [ ] Can you explain why node ID stability matters?

## Scope Limits & Safeguards

### ✅ MUST DO

- Document current reality (v2 implementation as-is)
- Provide concrete examples for every ID format
- Explain rationale for design decisions
- Link to existing documentation for details

### ❌ DO NOT

- Invent new node ID formats not currently used
- Change any code (this is documentation-only)
- Make breaking changes to existing ID formats
- Redesign the database architecture

### ⚠️ If Uncertain

- If you find inconsistencies between Python and Rust ID formats → document both and flag the discrepancy
- If you're unsure about an ID format → check the migration SQL and existing code
- If stability policy seems unclear → ask user for clarification

## Success Criteria

- [ ] `docs/architecture/data-architecture-v2.md` exists
- [ ] Document is 1500-2500 words (comprehensive but focused)
- [ ] All node ID formats documented with examples
- [ ] Stability policy clearly states guarantees
- [ ] 2-DB design rationale explained
- [ ] Graph update process documented
- [ ] Document reviewed for consistency with codebase
- [ ] No contradictions with existing documentation

## Related Files

**Read These Files:**
- `/docs/database-architecture/01-migration-history-from-mvp.md` - Historical context
- `/docs/database-architecture/02-current-state-assessment.md` - Current state
- `/docs/content-db-schema.md` - Schema details
- `/rust/crates/iqrah-core/src/domain/models.rs` - Domain models
- `/rust/crates/iqrah-storage/src/content/repository.rs` - Repository logic
- `/research_and_dev/iqrah-knowledge-graph2/src/iqrah/graph/knowledge_builder.py` - Python ID generation

**Create This File:**
- `/docs/architecture/data-architecture-v2.md` - New authoritative doc

**Reference in Future Tasks:**
- Task 1.3 (Node ID utility module) - Will implement parsing per this spec
- Task 1.5 (Stability validation) - Will enforce policy documented here
- Task 2.1 (Knowledge graph generation) - Will follow ID formats documented here

## Notes

This is a **foundational task**. The quality and clarity of this document will directly impact:
- Task 1.3 implementation (node ID parsing)
- Task 1.5 validation (stability checks)
- Task 2.1 data generation (graph with correct IDs)

Take time to be thorough and precise. When in doubt, favor explicitness over brevity.
