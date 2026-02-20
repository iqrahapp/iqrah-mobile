# 03 - Ticket Board: Core Learning And Scheduler

Source anchors:
1. `docs/ai_blueprint-3/04_scheduler_paths_current_vs_intended.md`
2. `docs/ai_blueprint-3/11_memorization_domain_constraints.md`
3. `docs/ai_blueprint-3/13_exercise_catalog_scored_and_runtime_wiring.md`
4. `docs/ai_blueprint-3/14_execution_backlog_file_level.md`

## Foundation

- [ ] `C-001` Add deterministic golden session scenarios (cold-start, due-review, chunk mode)
- [ ] `C-002` Add scheduler invariants test suite (duplicates/limits/axis mapping)

## Core Fixes

- [ ] `C-003` Fix cold-start empty-session behavior
  - Accept: brand-new user always gets non-empty session.

- [ ] `C-004` Make goal/chunk ID alter candidate selection
  - Accept: different goal/chunk inputs produce different candidate pools.

- [ ] `C-005` Implement 3-budget session composition
  - Budgets: continuity + due review + lexical understanding.

- [ ] `C-006` Add lexical fragility prioritization policy
  - Include frequency/spread/prayer-context weighting.

- [ ] `C-007` Promote core lexical exercises into scheduled default pool
  - Minimum: `mcq_ar_to_en`, `contextual_translation`, one cloze-MCQ continuity type, redesigned `identify_root`.

- [ ] `C-008` Demote high-friction low-ROI exercises from default scheduled pool
  - Keep optional/challenge modes where needed.

- [ ] `C-009` Enforce axis-to-exercise mapping guardrails in code and tests
- [ ] `C-010` Add session telemetry for budget mix and outcome quality

## Blueprint Gap Closures (Previously Missing)

- [ ] `C-011` Resolve CBOR import persistence gap (`G3`)
  - Scope:
    - either implement persisted node/edge import in `cbor_import.rs`,
    - or explicitly disable/deprecate fallback path with fail-fast behavior and docs update.
  - Accept:
    - no ambiguity remains about whether CBOR fallback persists data.

- [ ] `C-012` Restrict prerequisite parent query to dependency edges (`G6`)
  - Scope:
    - ensure prerequisite logic excludes non-dependency knowledge edges.
  - Accept:
    - tests prove dependency-only behavior.

- [ ] `C-013` Initialize unseen propagation targets safely (`G7`)
  - Scope:
    - propagation can create/update missing target states under controlled rules.
  - Accept:
    - first exposures produce measurable connected-progress impact.

## Exit Criteria

1. New user gets meaningful first session.
2. Sessions consistently contain all three learning budgets.
3. Scheduled exercises match intended pedagogical priorities.
4. G3/G6/G7 gaps are closed with tests and code-path evidence.
