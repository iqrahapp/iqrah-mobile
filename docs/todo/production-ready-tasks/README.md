# Production-Ready Data Architecture Tasks

This directory contains detailed task specifications for making the iqrah-mobile project production-ready, with focus on data architecture, knowledge graph integration, and system stability.

## Overview

**Total:** 15 tasks across 5 phases
**Estimated Timeline:** 3-4 weeks
**Architecture Decision:** 2-DB design (content.db + user.db)

### Key Objectives

1. **Standardize node ID handling** across Python and Rust
2. **Complete knowledge axis implementation** with full graph data
3. **Add data integrity safeguards** (versioning, transactions, validation)
4. **Enable package management** foundation (translator selection)
5. **Production-quality error handling**

## Task Execution Guidelines

### For AI Agents

Each task document is designed for **one AI agent session**. Before starting:

1. ✅ Read the entire task document thoroughly
2. ✅ Check dependencies are completed
3. ✅ Understand scope limits (DO NOT / MUST sections)
4. ✅ Plan verification approach before coding

### Verification Requirements

**Every task MUST include:**
- Unit tests (where applicable)
- Integration/CLI tests (mandatory)
- Manual verification steps (documented)

**Definition of Done:**
- All tests pass
- All verification steps completed
- Code follows project style (run `cargo fmt`, `cargo clippy`)
- Documentation updated (if applicable)

### CI Validation (MANDATORY)

Before marking task complete, run:

```bash
cd rust
RUSTFLAGS="-D warnings" cargo build --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo fmt --all -- --check
```

All must pass with zero warnings/errors. See [CLAUDE.md](/CLAUDE.md) for details.

## Phase Structure

### Phase 1: Critical Foundation (P0) - 1 Week

**Goal:** Establish stable architecture foundation

| Task | Description | Effort | Dependencies |
|------|-------------|--------|--------------|
| 1.1 | Document Architecture & Node ID Contracts | 1 day | None |
| 1.2 | Add Schema Versioning System | 1 day | None |
| 1.3 | Implement Node ID Utility Module | 1 day | None |
| 1.4 | Refactor Repository to Use Node ID Module | 2 days | 1.3 |
| 1.5 | Add Node ID Stability Validation (Python) | 1 day | None |

**Parallelization:** Tasks 1.1, 1.2, 1.3, 1.5 can run in parallel. Task 1.4 must wait for 1.3.

### Phase 2: Knowledge Axis Completion (P0) - 1 Week

**Goal:** Make knowledge axis fully functional with data

| Task | Description | Effort | Dependencies |
|------|-------------|--------|--------------|
| 2.1 | Generate Full Knowledge Graph with Axis Nodes | 2-3 days | 1.1 |
| 2.2 | Verify Knowledge Axis End-to-End Flow | 1 day | 2.1 |
| 2.3 | Implement Tajweed Exercise Type | 2 days | 2.1 |
| 2.4 | Add Cross-Axis Propagation Verification | 1 day | 2.1 |

**Parallelization:** Task 2.1 is blocking. After 2.1, tasks 2.2, 2.3, 2.4 can run in parallel.

### Phase 3: Data Integrity (P1) - 3 Days

**Goal:** Production-quality data handling

| Task | Description | Effort | Dependencies |
|------|-------------|--------|--------------|
| 3.1 | Add Transaction Wrapping to Review Recording | 1 day | None |
| 3.2 | Add Referential Integrity Validation | 1 day | 1.4 |
| 3.3 | Add Graph Update Mechanism | 1 day | 3.2 |

**Parallelization:** Tasks 3.1 and 3.2 can run in parallel. Task 3.3 depends on 3.2.

### Phase 4: Package Management Foundation (P1) - 4 Days

**Goal:** Enable content customization

| Task | Description | Effort | Dependencies |
|------|-------------|--------|--------------|
| 4.1 | Translator Selection UI & Preference | 3 days | None |
| 4.2 | Package Download Foundation (Design) | 1 day | None |

**Parallelization:** Both tasks can run in parallel.

### Phase 5: Production Hardening (P2) - 2 Days

**Goal:** Production-quality polish

| Task | Description | Effort | Dependencies |
|------|-------------|--------|--------------|
| 5.1 | Error Handling & Logging Audit | 2 days | All Phase 1-4 |

**Parallelization:** None (depends on all previous tasks).

## Dependency Graph

```
Phase 1:
  1.1 ─────┐
  1.2 ─────┤
  1.3 ──┬──┤
  1.5   │  │
        │  │
        v  v
       1.4 ─────┐
                │
                v
              Phase 2:
                2.1 ──┬── 2.2
                      ├── 2.3
                      └── 2.4
                      │
                      v
                    Phase 3:
                      3.1 ──┐
                      3.2 ──┼─ 3.3
                            │
                            v
                          Phase 4:
                            4.1
                            4.2
                            │
                            v
                          Phase 5:
                            5.1
```

## Critical Path

**Minimum path to functional knowledge axis:**
1.1 → 1.3 → 1.4 → 2.1 → 2.2

**Minimum path to production stability:**
Critical path above + 1.2 + 3.1 + 3.2 + 5.1

## Task Document Format

Each task follows this structure:

```markdown
# Task X.Y: [Name]

## Metadata
- **Priority:** P0/P1/P2
- **Estimated Effort:** X days
- **Dependencies:** [List]
- **Agent Type:** Research / Implementation / Testing

## Goal
Clear 1-2 sentence description

## Context
Why this task exists, background

## Current State
What exists today (file paths, line numbers)

## Target State
What should exist after completion

## Implementation Steps
1. Detailed step (with file paths)
2. ...

## Verification Plan
- [ ] Unit test: ...
- [ ] Integration test: ...
- [ ] Manual verification: ...

## Scope Limits & Safeguards
❌ DO NOT: ...
✅ MUST: ...
⚠️ If uncertain: Ask user

## Success Criteria
- [ ] Checklist item 1
- [ ] Checklist item 2

## Related Files
- `/path/to/file`: Description
```

## Progress Tracking

Update this section as tasks complete:

### Phase 1: Critical Foundation
- [ ] Task 1.1: Architecture Documentation
- [ ] Task 1.2: Schema Versioning
- [ ] Task 1.3: Node ID Utility Module
- [ ] Task 1.4: Repository Refactoring
- [ ] Task 1.5: Node ID Stability Validation

### Phase 2: Knowledge Axis Completion
- [ ] Task 2.1: Generate Full Knowledge Graph
- [ ] Task 2.2: Verify End-to-End Flow
- [ ] Task 2.3: Implement Tajweed Exercises
- [ ] Task 2.4: Cross-Axis Propagation

### Phase 3: Data Integrity
- [ ] Task 3.1: Transaction Wrapping
- [ ] Task 3.2: Referential Integrity
- [ ] Task 3.3: Graph Update Mechanism

### Phase 4: Package Management
- [ ] Task 4.1: Translator Selection UI
- [ ] Task 4.2: Package Download Design

### Phase 5: Production Hardening
- [ ] Task 5.1: Error Handling Audit

## Notes for Future Tasks

### Deferred to Post-MVP
- Full package download implementation (beyond design)
- Audio recitations system
- Full Quran knowledge graph (chapters 4-114)
- Performance optimization for large graphs
- Advanced analytics and reporting

### Known Technical Debt
- String-based node IDs (acceptable, but could be typed enums)
- No cross-DB foreign keys (mitigated by validation)
- Manual CBOR → SQL conversion (could be automated)

## Getting Help

If you encounter issues:
1. Check [docs/database-architecture/](/docs/database-architecture/) for design context
2. Review [CLAUDE.md](/CLAUDE.md) for CI requirements
3. Check git history: `git log --oneline --graph --all`
4. Ask user for clarification (don't guess)

## References

- Architecture audit: [docs/database-architecture/](/docs/database-architecture/)
- Content DB schema: [docs/content-db-schema.md](/docs/content-db-schema.md)
- Scheduler v2 design: [docs/todo/scheduler-v2-knowledge-graph.md](/docs/todo/scheduler-v2-knowledge-graph.md)
- R&D project: [research_and_dev/iqrah-knowledge-graph2/](/research_and_dev/iqrah-knowledge-graph2/)
