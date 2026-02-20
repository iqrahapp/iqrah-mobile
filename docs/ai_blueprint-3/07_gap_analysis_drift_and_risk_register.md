# 07 - Gap Analysis, Drift, And Risk Register

This section is the direct diagnosis of where the project drifted from the original intent.

## 1) High-Severity Gaps

### G1 - Runtime scheduler is due-only and goal-agnostic
- Evidence:
  - `SessionService::get_due_items(...)` only fetches due states.
  - `start_session(user_id, goal_id)` stores `goal_id` but selection ignores it.
- Impact:
  - weak onboarding from zero state
  - no controlled new-item introduction by curriculum
  - no lexical strategic focus

### G2 - Advanced scheduler path blocked by data readiness
- Evidence:
  - `goals` and `node_goals` are empty in shipped `rust/content.db`.
- Impact:
  - `scheduler_v2` cannot be adopted as-is in production path.

### G3 - CBOR import fallback is non-functional for persistence
- Evidence:
  - `cbor_import.rs` parses but insert calls are TODO/commented.
- Impact:
  - false confidence about runtime using fresh CBOR graph assets

### G4 - Graph shape drift between Python R&D and shipped runtime DB
- Evidence:
  - Python stats graph much denser than runtime DB.
  - runtime DB has no type-3 word nodes.
- Impact:
  - expected propagation/topology behavior from R&D may not appear in app.

## 2) Medium-Severity Gaps

### G5 - Inconsistent node typing metadata
- Evidence:
  - lemma IDs decode as type 7 but `nodes.node_type` stores 5.
- Impact:
  - SQL queries by `node_type` can misclassify lemmas as knowledge.

### G6 - `get_prerequisite_parents` includes knowledge edges
- Evidence:
  - edge filter uses `edge_type IN (0,1)`.
- Impact:
  - prerequisite gate could become noisy if scheduler_v2 is enabled.

### G7 - Propagation does not initialize unseen targets
- Evidence:
  - propagation updates only apply if target state already exists.
- Impact:
  - reduced visible "global progress" effect from first exposures.

### G8 - Exercise inventory and session policy are disconnected
- Evidence:
  - many exercise generators exist, but scheduled path uses a narrow subset.
- Impact:
  - potential pedagogical value remains untapped.

## 3) Low-Severity / Structural Debt

### G9 - Multiple graph-generation pipelines with unclear source-of-truth
- Python builder + Rust generator + CBOR path coexist.
- This creates maintenance and interpretation overhead.

### G10 - Legacy docs describe old paths
- Some architecture docs still reference outdated paths/behavior.

## 4) Overkill vs Underkill

### Overkill (for current shipped value)
1. Large simulation and bandit apparatus before stable production scheduler wiring.
2. Large exercise type surface without policy orchestration and quality gates.
3. Parallel graph build paths without strict contract parity tests.

### Underkill (where it hurts real memorization outcomes)
1. No strong onboarding/intro scheduling from cold start.
2. No first-class fragile-word/root prioritization in daily session plan.
3. No clear objective mix in session composition (verse flow vs lexical meaning vs root families).
4. No reliable runtime path from R&D graph insights into session behavior.

## 5) Bottom Line

The project has strong engineering ambition and many high-value components.
The central problem is integration order: advanced systems were built before locking a thin, measurable, user-value core loop.
