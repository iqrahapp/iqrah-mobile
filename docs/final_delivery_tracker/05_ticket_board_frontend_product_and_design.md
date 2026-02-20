# 05 - Ticket Board: Frontend Product And Design

Source anchors:
1. `docs/ai_blueprint-3/10_flutter_frontend_reality_and_gaps.md`
2. `docs/ai_blueprint-3/11_memorization_domain_constraints.md`
3. `docs/ai_blueprint-3/13_exercise_catalog_scored_and_runtime_wiring.md`

## Core Product Flows

- [ ] `F-001` Rebuild practice entry flow with explicit mode selection
  - Modes: daily mixed, chunk-focused, lexical-focused.

- [ ] `F-002` Implement chunk-mode UX (surah/range selection in <= 2 taps)
- [ ] `F-003` Improve session summary with clear learning-budget breakdown
- [ ] `F-004` Surface propagation impact as a first-class feedback moment

## Quran Reader Excellence

- [ ] `F-005` Deliver polished Quran reader flow (surah -> ayah -> word)
- [ ] `F-006` Word detail panel: root, meaning, occurrences, related forms
- [ ] `F-007` Fast navigation and continuity aids for memorization

## Exercise UX Quality

- [ ] `F-008` Redesign `identify_root` to low-friction interaction (no raw Arabic free text default)
- [ ] `F-009` Improve free-text grading UX and error messaging for translation-style exercises
- [ ] `F-010` Ensure scheduled exercise widgets are complete and stable for promoted core pool

## Audio And Motivation

- [ ] `F-011` Add audio-assisted practice loop in core session path
- [ ] `F-012` Add reciter-aware audio controls (foundation for multiple qaris)

## Visual Quality And Product Polish

- [ ] `F-013` Unify design system for a premium, cohesive interface
  - Scope-bound deliverables:
    - tokenized color/spacing/type scale,
    - component specs for: buttons, cards, list rows, input fields, exercise containers, progress chips,
    - applied to: dashboard, practice entry, exercise page, session summary, reader pages.
  - Done when:
    - no page in the core flow uses ad-hoc styling outside design tokens/components.
  - Non-goals:
    - no full design-system expansion beyond core learning flow pages in this ticket.

- [ ] `F-014` Production polish pass
  - responsive mobile layouts,
  - loading/empty/error states,
  - motion and transitions,
  - accessibility baseline.

## Exit Criteria

1. UI feels intentional and premium, not utilitarian/debug-like.
2. Reader and session flows align with real memorization practice.
3. Users clearly see progress, meaning gains, and graph-wide impact.
