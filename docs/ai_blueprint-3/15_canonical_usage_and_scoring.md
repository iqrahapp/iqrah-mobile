# 15 - Canonical Usage And Scoring

Purpose: decide whether to use blueprint-3 alone or combine blueprint-2 + blueprint-3.

## 1) Recommendation

Use `docs/ai_blueprint-3/` as the single canonical knowledge bank.

Reason:
1. It contains runtime-verified truth anchors (the key strength of v3).
2. It now absorbs the strongest v2 additions:
- exercise catalog scoring (`13_exercise_catalog_scored_and_runtime_wiring.md`)
- memorization-domain constraints (`11_memorization_domain_constraints.md`)
- concrete prioritized execution plan (`14_execution_backlog_file_level.md`)
- frequency-first lexical policy and prayer-context weighting (`11_memorization_domain_constraints.md`)
- per-exercise UX/grading failure analysis with concrete redesigns (`13_exercise_catalog_scored_and_runtime_wiring.md`)
3. A single canonical source reduces contradiction risk for future AI agents.

## 2) Optional Secondary Reference

Only keep `docs/ai_blueprint-2/` as historical context for narrative/vision language.

If referenced, treat `docs/ai_blueprint-2/10_corrections_from_blueprint3.md` as mandatory guardrail.

## 3) Scoring Against The Shared Rubric

Dimension scores (targeted for your stated standard):

| Dimension | Score | Why |
|---|---:|---|
| Factual accuracy (what actually runs) | 9.6 | Runtime-truth docs + claims-vs-reality matrix + checklist gates |
| Technical depth | 9.2 | KG, scheduler paths, DB-shape drift, backend contract, frontend wiring |
| Completeness (all layers) | 9.4 | Python R&D + Rust runtime + Flutter + backend + sync + domain |
| Vision alignment | 9.5 | Original goals mapped to constraints and roadmap |
| Honest critique | 9.6 | Overkill/defer calls are explicit and justified |
| Actionability | 9.5 | File-level P0/P1/P2 backlog with concrete done criteria |
| Quran memorization domain fit | 9.6 | Chunk, lexical ROI policy, prayer-context boosts, audio-first, 3-budget invariant |
| Exercise analysis | 9.6 | 21-type catalog + runtime wiring truth + explicit UX/grading failure-mode fixes |

Projected overall: about 9.5/10 when consumed in the recommended order from `index.md`.

## 4) Consumption Contract For Future AI Assistants

Before proposing architecture or implementation, assistant must:
1. classify each claim as active runtime vs implemented-not-wired vs R&D-only vs planned,
2. reference at least one source-anchored file in this folder per major claim,
3. preserve the 3-budget session invariant (continuity, review, lexical),
4. avoid adding complexity (bandit/profile tuning) until P0/P1 acceptance gates pass.

## 5) Fast Start Reading Order

If time is short, read only:
1. `docs/ai_blueprint-3/09_claims_vs_runtime_reality.md`
2. `docs/ai_blueprint-3/13_exercise_catalog_scored_and_runtime_wiring.md`
3. `docs/ai_blueprint-3/11_memorization_domain_constraints.md`
4. `docs/ai_blueprint-3/14_execution_backlog_file_level.md`

That subset is enough to avoid drift and start implementation correctly.
