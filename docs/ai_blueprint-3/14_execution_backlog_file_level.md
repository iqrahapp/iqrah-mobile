# 14 - Execution Backlog (File-Level, AI-Ready)

Purpose: convert this audit into a concrete build sequence another AI can execute without drifting.

## 0) North-Star Invariant

Each 5-15 minute session must include:
1. continuity work (chunk/surah flow),
2. due review work (retention),
3. lexical work (word/root meaning on fragile high-value items).

If any one of the three is missing, the session is not acceptable.

## 1) P0 - Fix Runtime Learning Loop

### P0.1 Cold-start session must never be empty

Current issue:
- New user with no `user_memory_states` can receive zero due items.

Primary files:
- `rust/crates/iqrah-core/src/services/session_service.rs`
- `rust/crates/iqrah-api/src/api.rs`

Implementation target:
- If due queue is below minimum size, backfill from scoped intro candidates (verse/word/root mix).
- Persist initial states so propagation and FSRS can operate immediately.

Done when:
1. New user can always start and complete a non-empty session.
2. Session produces persisted state changes in local DB.

### P0.2 Goal-aware selection must be real, not metadata-only

Current issue:
- `goal_id` is stored in session record but not used for candidate selection.

Primary files:
- `rust/crates/iqrah-api/src/api.rs`
- `rust/crates/iqrah-core/src/services/session_service.rs`
- `rust/crates/iqrah-storage/src/repositories/` (goal-aware queries if needed)

Implementation target:
- Candidate query and ranking are scoped by selected goal/chunk/surah when provided.
- Fallback to global daily mode only when no explicit scope exists.

Done when:
1. Two different goal IDs produce different candidate pools.
2. Scope effect is test-covered and observable from app behavior.

### P0.3 Promote core lexical exercises into default scheduling

Current issue:
- Default scheduled path uses narrow subset; many high-value lexical exercises are sandbox-only.

Primary files:
- `rust/crates/iqrah-core/src/exercises/service.rs`
- `rust/crates/iqrah-core/src/services/session_service.rs`
- `lib/features/exercises/widgets/`
- `lib/pages/exercise_page.dart`

Promote first:
1. `mcq_ar_to_en`
2. `contextual_translation`
3. `missing_word_mcq` or `next_word_mcq`
4. `identify_root`

Done when:
1. Scheduled sessions include lexical exercises by policy, not chance.
2. Widget routing exists and is stable for promoted types.

## 2) P1 - Product Fit for Real Memorization

### P1.1 Add explicit chunk mode in UI and planner

Primary files:
- `lib/pages/practice_page.dart`
- `lib/providers/session_provider.dart`
- `rust/crates/iqrah-api/src/api.rs`
- `rust/crates/iqrah-core/src/services/session_service.rs`

Implementation target:
- User picks surah/range/chunk.
- Planner enforces chunk continuity budget while still respecting due reviews.

Done when:
1. User can run chunk-focused session in under 2 taps.
2. Session summary clearly separates chunk progress vs global review.

### P1.2 Surface propagation impact in post-answer and summary UX

Primary files:
- `lib/features/session/`
- `lib/pages/session_summary_page.dart`
- `rust/crates/iqrah-api/src/api.rs` (if additional payload needed)

Implementation target:
- After key answers, show "this reinforced N connected words/verses/roots".
- Summary includes propagated impact totals.

Done when:
1. User sees graph-wide impact as a first-class feedback loop.
2. Numbers reflect real DB writes, not static estimates.

### P1.3 Audio-assisted practice path in main flow

Primary files:
- `lib/pages/exercise_page.dart`
- `lib/services/media/` (or existing media path)
- Rust API layer if extra audio metadata is required

Implementation target:
- At least one session mode supports listen -> recall/repeat loop.

Done when:
1. Audio-assisted exercise is part of normal session flow.
2. Works offline for bundled content.

## 3) P2 - Hardening and Scaling

### P2.1 Graph/runtime parity checks

Primary files:
- `research_and_dev/iqrah-knowledge-graph2/`
- `rust/crates/iqrah-gen/`
- `rust/content.db` validation scripts/tests

Implementation target:
- Automated checks for node/edge type distribution, metadata key completeness, and goal table readiness.

Done when:
1. CI or local audit script flags schema/content drift before shipping.

### P2.2 Backend sync reliability verification

Primary files:
- `/home/shared/ws/iqrah/iqrah-backend/openapi.json`
- `lib/providers/sync_provider.dart`
- `lib/services/sync_service.dart`
- `lib/services/sync_mapper.dart`

Implementation target:
- Two-device conflict scenario is tested and documented.

Done when:
1. Push/pull behavior is deterministic for conflicting updates.
2. Failure states are visible and recoverable in app UI.

## 4) Acceptance Gates (Must Pass Before Major Refactor)

1. Runtime truth gate:
- Re-verify scheduler path, goal usage, and exercise routing against current code.

2. User-loop gate:
- New install -> first session -> summary -> second session works without DB/manual intervention.

3. Memorization-fit gate:
- Session telemetry confirms all three budgets: continuity, review, lexical.

4. Frontend gate:
- Word tap flow exposes root + meaning + cross-occurrence context.

## 5) Suggested Work Sequence (Minimal Risk)

1. Ship P0.1 and P0.2 together.
2. Ship P0.3 with UI routing checks.
3. Ship P1.1 and P1.2 as one "product feel" release.
4. Add P1.3 audio loop.
5. Then run P2 hardening.

This sequence maximizes user-visible improvement early while reducing architecture drift risk.
