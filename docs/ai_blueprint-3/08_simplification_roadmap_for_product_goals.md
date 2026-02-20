# 08 - Simplification Roadmap Aligned To Product Goals

This roadmap is designed to recover momentum and align implementation with your original 7 goals.

## 1) Goal Alignment Snapshot

### Goal 1 - Mobile app
Status: achieved baseline, quality uneven.

### Goal 2 - Fast local performance
Status: mostly achieved by local SQLite + local Rust logic.

### Goal 3 - Backup/sync across devices
Status: foundational path exists (auth + push/pull), still maturing.

### Goal 4 - Quran explorer depth (surah/ayah/word/root)
Status: partial. Data exists, but end-to-end UX consistency is incomplete.

### Goal 5 - Multi-language and multiple qaris
Status: schema support exists; product-level integration appears partial.

### Goal 6 - Memorization + understanding for busy non-Arab users
Status: partially achieved. Core loop currently too generic and not lexically strategic.

### Goal 7 - Progress propagation from connected units
Status: concept implemented partially; runtime effect weakened by state initialization and graph/runtime drift.

## 2) What To Trim Now

1. Freeze bandit/profile experimentation in production path until baseline scheduler KPIs are stable.
2. Reduce active exercise set in scheduled sessions to a curated core tied to clear outcomes.
3. Stop relying on CBOR fallback narrative until import persistence is implemented.
4. Declare one production graph source-of-truth and demote others to R&D-only until parity tests exist.

## 3) What To Strengthen Immediately

1. Cold-start scheduling path
- from first session, schedule meaningful new items
- do not depend on pre-existing due states

2. Lexical fragility loop
- track per-word/per-root error burden
- guarantee lexical reinforcement quota each session

3. Goal-aware planning
- use explicit goal scopes (surah/juz/cluster)
- define intro budget + review budget + lexical budget

4. Propagation visibility
- show user-level "you reinforced X related items" feedback
- initialize target states when safe, not only update existing states

## 4) Suggested 3-Phase Execution

### Phase A (stabilize core loop)
- Wire a single production session planner path.
- Add deterministic onboarding from zero.
- Add telemetry for intro/review/lexical mix and recall outcomes.

### Phase B (pedagogy fit)
- Introduce lexical/root prioritization policy.
- Use 3-way session composition:
  - verse continuity
  - word recall/meaning
  - root-family reinforcement
- Keep exercise set intentionally small but high-quality.

### Phase C (scale + sync hardening)
- Tighten sync conflict semantics and observability.
- Add parity tests between graph generator output and runtime scheduler expectations.
- Re-enable advanced optimizers (bandit, richer scheduling) only after baseline metrics are reliable.

## 5) Concrete Architectural Decision

For now, optimize for one clear invariant:

"Every 10-minute session must produce measurable memorization progress and lexical understanding progress, even for a new user with zero prior states."

Any subsystem that does not directly improve this invariant should be considered optional until the invariant is proven in production telemetry.
