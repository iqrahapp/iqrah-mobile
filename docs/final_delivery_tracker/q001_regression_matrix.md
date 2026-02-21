# Q-001 - Cross-Repo Regression Matrix

**Ticket:** Q-001 `[BOTH]`
**Status:** Complete (MOB portion)
**Date:** 2026-02-21
**Scope:** core flow · bootstrap · sync

---

## Purpose

This matrix defines the minimum regression coverage required before any release or significant refactor across `iqrah-mobile` and `iqrah-backend`. Each scenario has a defined entry point, expected outcome, and the test file that covers it.

Use this matrix as a pre-merge checklist and as the canonical source for which tests must be green for Gate A, B, and C to pass.

---

## Coverage Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Covered by automated test |
| ⚠️ | Partially covered or manual only |
| ❌ | Not yet covered — needs test |
| `[MOB]` | iqrah-mobile only |
| `[BE]` | iqrah-backend only |
| `[BOTH]` | Both repos must pass |

---

## 1. Core Flow — Session Generation

These scenarios cover the scheduler's ability to produce valid, non-empty, pedagogically coherent sessions.

| ID | Scenario | Repo | Coverage | Test File / Location | Notes |
|----|----------|------|----------|----------------------|-------|
| CF-01 | Cold-start: brand-new user receives non-empty first session | `[MOB]` | ❌ | `rust/crates/iqrah-cli/tests/scheduler_integration.rs::test_scheduler_with_new_user` only asserts candidate pool size (493) and empty memory states — does NOT call the session generator or assert a non-empty session output | Ticket C-003 |
| CF-02 | Goal metadata is correct (label, type, verse count) | `[MOB]` | ✅ | `rust/crates/iqrah-cli/tests/scheduler_integration.rs::test_scheduler_goal_data` | 493-verse goal validated |
| CF-03 | Due-review: returning user with expired reviews gets review-heavy session | `[MOB]` | ❌ | Not yet written — needed for C-003 | Ticket C-003 |
| CF-04 | Chunk-mode: different chunk/goal IDs produce different candidate pools | `[MOB]` | ❌ | Not yet written — needed for C-004 | Ticket C-004 |
| CF-05 | Session composition contains all three budgets (continuity / due-review / lexical) | `[MOB]` | ❌ | Not yet written — needed for C-005 | Ticket C-005 |
| CF-06 | Axis-to-exercise mapping never produces an invalid pairing | `[MOB]` | ⚠️ | `rust/crates/iqrah-core/src/services/session_service.rs::test_invariant_axis_filter_excludes_all_non_matching` (partial) | Needs guardrail assertions in code path (C-009) |
| CF-07 | Duplicate exercises never appear in the same session | `[MOB]` | ✅ | `rust/crates/iqrah-core/src/services/session_service.rs::test_invariant_no_duplicate_node_ids` | Covered by C-002 invariants |
| CF-08 | Session size respects configured limits | `[MOB]` | ✅ | `rust/crates/iqrah-core/src/services/session_service.rs::test_invariant_limit_never_exceeded` + `::test_respects_limit` | Covered by C-002 |
| CF-09 | Backend session-start endpoint records session correctly | `[BE]` | ❌ | Backend integration test — not yet written | |
| CF-10 | Backend session-end endpoint persists outcome and triggers sync | `[BE]` | ❌ | Backend integration test — not yet written | |

---

## 2. Bootstrap — Artifact Loading & Initialization

These scenarios cover the loading of `content.db` and the knowledge graph into the Rust runtime.

| ID | Scenario | Repo | Coverage | Test File / Location | Notes |
|----|----------|------|----------|----------------------|-------|
| BS-01 | Content DB initialises with all migrations applied | `[MOB]` | ✅ | `rust/crates/iqrah-storage/tests/integration_tests.rs::test_content_db_initialization` | |
| BS-02 | User DB initialises with correct schema version (2.0.0) | `[MOB]` | ✅ | `rust/crates/iqrah-storage/tests/integration_tests.rs::test_user_db_initialization_and_migrations` | |
| BS-03 | Sample data present after init: Chapter 1, Verse 1:1, 4 words | `[MOB]` | ✅ | `rust/crates/iqrah-storage/tests/integration_tests.rs::test_content_repository_crud` | |
| BS-04 | CBOR knowledge graph loads without error | `[MOB]` | ❌ | `rust/crates/iqrah-core/src/cbor_import.rs` has NO tests; import persistence is commented-out TODO — `insert_nodes_batch` / `insert_edges_batch` not yet called | C-011 required |
| BS-05 | CBOR import: all node IDs resolvable in content.db after import | `[MOB]` | ❌ | Not yet written — gap G3 | Ticket C-011 |
| BS-06 | Knowledge axis initialises correctly from imported graph | `[MOB]` | ⚠️ | `rust/crates/iqrah-storage/tests/knowledge_axis_integration_test.rs` | Uses live `~/.local/share/iqrah/content.db`; not hermetic |
| BS-07 | Atomic activation: incomplete download does not replace live artifact | `[MOB]` | ❌ | Not yet written — needed for D-012 or D-013 | |
| BS-08 | Rollback: failed activation restores previous artifact state | `[MOB]` | ❌ | Not yet written — needed for D-014 | |
| BS-09 | Checksum mismatch halts activation (does not corrupt live DB) | `[BOTH]` | ❌ | Not yet written — Q-005 | |
| BS-10 | Backend release registry returns valid artifact URLs and checksums | `[BE]` | ❌ | Backend integration test — not yet written | |
| BS-11 | Mobile fetch + checksum verification succeeds end-to-end (happy path) | `[BOTH]` | ❌ | E2E test — not yet written | |

---

## 3. Sync — User Progress Backup & Restore

These scenarios cover cross-device and cloud sync of user memory states.

| ID | Scenario | Repo | Coverage | Test File / Location | Notes |
|----|----------|------|----------|----------------------|-------|
| SY-01 | Memory states persist correctly in user.db (CRUD round-trip) | `[MOB]` | ✅ | `rust/crates/iqrah-storage/tests/integration_tests.rs::test_user_repository_memory_states` | |
| SY-02 | Node ID integrity: all memory states reference valid content nodes | `[MOB]` | ⚠️ | `iqrah-cli integrity` command covers runtime check; no automated test | |
| SY-03 | Content DB update compatibility: old node IDs present in new DB | `[MOB]` | ✅ | `rust/crates/iqrah-cli/src/verify_update.rs` (unit tests inside file) | |
| SY-04 | Sync upload: user.db snapshot reaches backend without data loss | `[BOTH]` | ❌ | Not yet written — D-018 / D-019 | |
| SY-05 | Sync download: backend snapshot restores correctly on new device | `[BOTH]` | ❌ | Not yet written — D-020 | |
| SY-06 | Conflict resolution: last-write-wins with timestamp is correct | `[BOTH]` | ❌ | Not yet written — D-022 | |
| SY-07 | Offline learning: session completes and queues sync for later | `[MOB]` | ❌ | Not yet written — D-021 | |
| SY-08 | Version compatibility: backend rejects stale client schema version | `[BOTH]` | ❌ | Not yet written — D-019 | |
| SY-09 | Backend sync endpoint rate-limited (no abuse vector) | `[BE]` | ❌ | Not yet written — Q-007 | |

---

## 4. Summary — Coverage by Gate

| Gate | Required Scenarios | Currently Covered | Gap |
|------|--------------------|-------------------|-----|
| **Gate A** (core learning) | CF-01..08 | CF-02, CF-07, CF-08 | CF-01 (no session-generation assertion), CF-03..06 missing (C-003..C-009) |
| **Gate B** (data platform) | BS-04..11 | BS-01..03 (infra only) | BS-04..11 all missing; BS-04 has no test and import is non-persistent |
| **Gate C** (sync/backup) | SY-01..09 | SY-01, SY-03 | SY-02, SY-04..09 missing |

---

## 5. How To Run Mobile Regression Suite

```bash
cd rust

# Full suite
cargo test --all-features

# Scheduler integration only
cargo test --package iqrah-cli

# Storage integration only
cargo test --package iqrah-storage

# Core only
cargo test --package iqrah-core
```

All commands must be run with zero failures before any merge to `main`.

---

## 6. Backend Regression Scenarios (To Be Implemented in `iqrah-backend`)

> The backend engineer must implement these scenarios in `iqrah-backend` and link them here. Commands come from `01_execution_protocol.md`.

```bash
cd ../iqrah-backend

just test          # all unit + integration tests
just coverage-ci   # coverage gate
just spec-check    # OpenAPI spec consistency
```

### Backend Scenarios (CF, BS, SY entries marked `[BE]` or `[BOTH]`)

| ID | Scenario | Expected Behaviour | Where to Test |
|----|----------|--------------------|---------------|
| CF-09 | Session-start endpoint records session | HTTP 200, session row created, user_id + started_at stored | `iqrah-backend` integration test |
| CF-10 | Session-end endpoint persists outcome | HTTP 200, memory state updated, sync event emitted | `iqrah-backend` integration test |
| BS-10 | Release registry returns artifact manifest | HTTP 200, JSON with `url`, `checksum`, `version` fields | `iqrah-backend` integration test |
| BS-11 | Mobile fetches artifact end-to-end (happy path) | Checksum matches, content activatable | Cross-repo E2E (post Gate B) |
| SY-04 | Sync upload: snapshot reaches backend | HTTP 200, snapshot stored, version incremented | `iqrah-backend` integration test |
| SY-05 | Sync download: snapshot restores on new device | HTTP 200, progress intact, no data loss | `iqrah-backend` integration test |
| SY-06 | Conflict: last-write-wins by timestamp | Later timestamp wins, earlier discarded | `iqrah-backend` unit test |
| SY-08 | Stale schema version rejected | HTTP 409 / HTTP 422 returned | `iqrah-backend` integration test |
| SY-09 | Sync endpoint rate-limited | HTTP 429 after N requests per minute | `iqrah-backend` integration test |

All scenarios must pass before Gate B/C sign-off.

---

## 7. Residual Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| BS-06 knowledge axis test uses live file path | High | Hermetise the test (use fixture DB) as part of C-011 |
| No end-to-end bootstrap test exists | High | Required before Gate B — D-012 to D-014 |
| Sync scenarios are entirely untested | High | D-018 to D-022 must deliver coverage before Gate C |
| Backend regression suite not yet defined | Medium | Backend engineer must mirror this matrix in `iqrah-backend` |

---

## 8. Linked Follow-Up Tickets

- `C-003` — cold-start session fix (CF-03)
- `C-004` — goal/chunk selection (CF-04)
- `C-005` — 3-budget composition (CF-05)
- `C-009` — axis mapping guardrails (CF-06)
- `C-011` — CBOR import persistence (BS-04, BS-05)
- `D-012..D-014` — atomic activation and rollback (BS-07, BS-08)
- `D-018..D-022` — sync scenarios (SY-04..SY-08)
- `Q-005` — checksum enforcement (BS-09)
- `Q-007` — backend rate limits (SY-09)
