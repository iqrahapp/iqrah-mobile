# AI Agent (Jules) Task Execution Prompt Template

Use this template to assign production-ready tasks to AI agents. Simply replace `[TASK-ID]` with the actual task filename.

---

## Prompt Template

```markdown
# Task Assignment: [TASK-ID]

Please execute the production-ready task documented in:
`/home/shared/ws/iqrah/iqrah-mobile/docs/todo/production-ready-tasks/[TASK-ID].md`

## Instructions

1. **Read the task document carefully** - It contains complete specifications, acceptance criteria, and implementation steps
2. **Follow the document exactly** - All requirements, scope limits, and safeguards are clearly defined
3. **Use the verification plan** - The document includes checklists to verify your work
4. **Respect scope limits** - The "DO NOT" section lists what to avoid
5. **Check dependencies** - Ensure prerequisite tasks are complete before starting

## Important Notes

- **Node IDs: Internal Ints, External Strings**
  - The architecture uses **INTEGER** IDs internally for all graph operations.
  - External APIs and user-facing data (`user.db`) use stable **STRING** unique keys (`ukeys`).
  - The `NodeRegistry` is the boundary layer that maps between `ukeys` and `integer IDs`.
  - **Exception**: `user.db` **always** uses string `ukeys` for stability across content updates.

- **Graph Operations: Always Use Integer IDs**
  - When querying the graph, always resolve the string `ukey` to an integer `id` first. This is critical for performance and referential integrity.
  - **❌ AVOID**: String-based queries, which are slow and deprecated.
    ```rust
    // This is incorrect and will not work with the new schema.
    query!("SELECT * FROM edges WHERE source_id = ?", "VERSE:1:1:memorization");
    ```
  - **✅ PREFER**: Integer-based queries for all graph operations.
    ```rust
    // Correct approach: resolve the ukey to an integer ID first.
    let node_id = registry.get_id("VERSE:1:1:memorization").await?;
    query!("SELECT * FROM edges WHERE source_id = ?", node_id);
    ```

- **Before committing:** Run ALL pre-commit checks from `CLAUDE.md`:
  ```bash
  cd rust
  RUSTFLAGS="-D warnings" cargo build --all-features
  cargo clippy --all-features --all-targets -- -D warnings
  cargo test --all-features
  cargo fmt --all -- --check
  ```

- **Python project:** Located at `research_and_dev/iqrah-knowledge-graph2/`
  - CLI command: `python -m iqrah_cli` (NOT `python -m iqrah.cli`)

- **Knowledge axes:** 6 total
  - Verse-level (4): memorization, translation, tafsir, tajweed
  - Word-level (2): contextual_memorization, meaning

## Success Criteria

Your work is complete when:
- [ ] All acceptance criteria in the task document are met
- [ ] All verification checklist items pass
- [ ] Pre-commit CI checks pass locally
- [ ] No scope limits violated
- [ ] Changes follow project architecture (see task-1.1)

## Questions?

If anything is unclear or you encounter blockers, ask for clarification before proceeding.

---

**Ready to start? Read the task document and begin implementation!**
```

---

## Usage Examples

### Example 1: Assign Task 2.1

```markdown
# Task Assignment: task-2.1-generate-full-knowledge-graph

Please execute the production-ready task documented in:
`/home/shared/ws/iqrah/iqrah-mobile/docs/todo/production-ready-tasks/task-2.1-generate-full-knowledge-graph.md`

[... rest of template ...]
```

### Example 2: Assign Task 1.3

```markdown
# Task Assignment: task-1.3-node-id-utility-module

Please execute the production-ready task documented in:
`/home/shared/ws/iqrah/iqrah-mobile/docs/todo/production-ready-tasks/task-1.3-node-id-utility-module.md`

[... rest of template ...]
```

### Example 3: Assign Task 2.2

```markdown
# Task Assignment: task-2.2-verify-knowledge-axis-end-to-end

Please execute the production-ready task documented in:
`/home/shared/ws/iqrah/iqrah-mobile/docs/todo/production-ready-tasks/task-2.2-verify-knowledge-axis-end-to-end.md`

[... rest of template ...]
```

---

## Quick Reference: All Available Tasks

### Phase 1: Foundation
- `task-1.1-architecture-documentation.md`
- `task-1.2-schema-versioning.md`
- `task-1.3-node-id-utility-module.md`
- `task-1.4-repository-refactoring.md`
- `task-1.5-node-id-stability-validation.md`

### Phase 2: Knowledge Graph
- `task-2.1-generate-full-knowledge-graph.md` (blocks 2.2, 2.3, 2.4)
- `task-2.2-verify-knowledge-axis-end-to-end.md` (after 2.1)
- `task-2.3-implement-tajweed-exercises.md` (after 2.1)
- `task-2.4-cross-axis-propagation-verification.md` (after 2.1)

### Phase 3: Data Integrity
- `task-3.1-transaction-wrapping.md`
- `task-3.2-referential-integrity-validation.md`
- `task-3.3-graph-update-mechanism.md`

### Phase 4: UX Features
- `task-4.1-translator-selection-ui.md`
- `task-4.2-package-download-design.md`

### Phase 5: Polish
- `task-5.1-error-handling-audit.md`

---

## Task Parallelization Guide

### Can Run in Parallel

**Phase 1 (4 tasks in parallel):**
- Task 1.1, 1.2, 1.3, 1.5 (all independent)

**Phase 2 (3 tasks in parallel after 2.1):**
- Task 2.2, 2.3, 2.4 (all depend on 2.1 completing first)

**Phase 3 (2 tasks in parallel):**
- Task 3.1 and 3.2 (both independent)

**Phase 4 (2 tasks in parallel):**
- Task 4.1 and 4.2 (both independent)

### Must Run Sequentially

- Task 1.4 depends on 1.3 completing
- Task 3.3 depends on 3.2 completing
- Task 2.1 blocks all Phase 2 tasks
- Task 5.1 should be last (final polish)
