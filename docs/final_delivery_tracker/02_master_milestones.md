# 02 - Master Milestones

## Milestone Board

- [ ] `M0` Runtime Truth + Safety Baseline
  - Objective: lock runtime behavior, test baselines, and no-regression harness.
  - Required tickets:
    - `C-001` `C-002` `Q-001` `Q-013` `Q-014`

- [ ] `M1` Core Learning Loop Reliability
  - Objective: non-empty cold-start, goal/chunk-aware planning, meaningful session composition.
  - Required tickets:
    - `C-003`..`C-013`

- [ ] `M2` Data Platform Release System
  - Objective: backend-hosted release model and artifact distribution foundation.
  - Required tickets:
    - `D-000`..`D-010`

- [ ] `M3` Mobile Bootstrap + Offline Robustness
  - Objective: remote artifact bootstrap with atomic activation and rollback safety.
  - Required tickets:
    - `D-011`..`D-017`

- [ ] `M4` Product UX + Beautiful Interface
  - Objective: polished Quran reader, superior exercise UX, propagation visibility, audio-first loop.
  - Required tickets:
    - `F-001`..`F-014`
  - Start condition:
    - may start immediately after `M1`; does not wait for `M2`/`M3` unless a specific `F` ticket declares a `D` dependency.

- [ ] `M5` Sync/Backup Production Readiness
  - Objective: reliable multi-device sync with observability and conflict confidence.
  - Required tickets:
    - `D-018`..`D-022` and `Q-006`

- [ ] `M6` Production Hardening + Launch Gate
  - Objective: performance, security, release readiness, operational playbooks.
  - Required tickets:
    - `Q-002`..`Q-012`

## Cross-Milestone Dependencies

1. Complete `M2` before `M3` default rollout.
2. `M4` runs in parallel with `M2`/`M3` after `M1`.
3. Complete core tickets `C-003` and `C-004` before any UI polish claims.
4. Complete `F-005` and `F-006` before calling memorization UX "production-ready".
5. Complete `Q-008` and `Q-009` before production launch.

## Authoritative Queue

Use `docs/final_delivery_tracker/08_exact_execution_order.md` as the strict run sequence.
