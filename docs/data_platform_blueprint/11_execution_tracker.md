# 11 - Execution Tracker

Status key:
- `[ ]` not started
- `[-]` in progress
- `[x]` done
- `[!]` blocked

## Milestone Board

- [ ] `MS-0` Baseline Stabilization
  - Scope: freeze runtime truth, stabilize CI, confirm current pack/sync APIs stay backward-compatible.
  - Gate:
    - backend CI green
    - mobile CI green
    - no breaking OpenAPI changes

- [ ] `MS-1` Backend Release Registry (Phase 1)
  - Scope: release schema + repository + admin/public release endpoints.
  - Tickets: `B-001` to `B-008`
  - Gate:
    - release create/attach/validate/publish works in tests
    - `openapi.json` updated and validated

- [ ] `MS-2` Backend Hardening + Admin CLI (Phase 2)
  - Scope: validation hardening, audit logs, CLI flows.
  - Tickets: `B-009` to `B-014`
  - Gate:
    - CLI can run full release lifecycle
    - publish blocks invalid releases

- [ ] `MS-3` Mobile Bootstrap Dual Mode (Phase 3)
  - Scope: remote release bootstrap behind feature flag.
  - Tickets: `M-001` to `M-007`
  - Gate:
    - fresh install can bootstrap required artifacts from backend
    - local fallback path still works

- [ ] `MS-4` Mobile Default Remote + Atomic Activation (Phase 4)
  - Scope: remote bootstrap default, fallback on failure, telemetry.
  - Tickets: `M-008` to `M-013`
  - Gate:
    - interrupted download recovery works
    - checksum mismatch never activates bad artifacts

- [ ] `MS-5` Repo Cleanup + Cutover (Phase 5)
  - Scope: remove heavy generated artifacts from default repo flow.
  - Tickets: `M-014`, `M-015`, `B-015`
  - Gate:
    - docs and onboarding updated
    - generated heavy artifacts no longer required in repo

## Cross-Repo Dependency Order

1. Complete `B-001`..`B-008` before `M-003`.
2. Complete `B-009`..`B-011` before `M-008`.
3. Complete `M-001`..`M-007` before removing bundled default path.
4. Complete `M-008`..`M-013` before cutover cleanup tickets.

## Active Queue (Fill During Execution)

- [ ] Current backend ticket:
- [ ] Current mobile ticket:
- [ ] Current blocker:
- [ ] Next unblock action:

## Required Completion Evidence Per Ticket

Each ticket is only `done` when all are provided:
1. Changed file list.
2. Test commands and results.
3. API or schema diff summary (if applicable).
4. Explicit statement of remaining risks.
