# 04 - Ticket Board: Data Platform, Backend, Sync

Source anchors:
1. `docs/data_platform_blueprint/12_tickets_backend.md`
2. `docs/data_platform_blueprint/13_tickets_mobile.md`
3. `docs/ai_blueprint-3/06_backend_scope_and_sync_contract.md`
4. `docs/ai_blueprint-3/09_claims_vs_runtime_reality.md`

## Data-Blueprint Verification Gate

- [ ] `D-000` Audit and score `docs/data_platform_blueprint` against current code/runtime
  - Deliverables:
    - short scorecard (accuracy, completeness, actionability),
    - corrections list,
    - ticket updates before `D-001` starts if needed.
  - Accept:
    - no `D` implementation proceeds on unverified assumptions.

## Backend Release Registry

- [ ] `D-001` Add release schema migrations (`dataset_releases`, `dataset_release_artifacts`)
- [ ] `D-002` Add release domain models and DTOs
- [ ] `D-003` Add `ReleaseRepository` + tests
- [ ] `D-004` Add admin release create/attach endpoints
- [ ] `D-005` Add release validation endpoint + rule engine
- [ ] `D-006` Add release publish endpoint (validation-gated)
- [ ] `D-007` Add public release endpoints (`latest`, `manifest`)
- [ ] `D-008` Add OpenAPI coverage + regenerate spec
- [ ] `D-009` Add release deprecate endpoint
- [ ] `D-010` Add admin audit logging for release actions

## Mobile Bootstrap Integration

- [ ] `D-011` Add release manifest client/service in mobile
- [ ] `D-012` Add resumable downloader with checksum verification
- [ ] `D-013` Add local release registry persistence
- [ ] `D-014` Add atomic activation manager
- [ ] `D-015` Add feature-flagged remote bootstrap startup path
- [ ] `D-016` Make remote bootstrap default for non-dev builds
- [ ] `D-017` Add rollback + retry safety for failed updates

## Sync/Backup Hardening

- [ ] `D-018` Add two-device conflict integration tests
- [ ] `D-019` Add sync observability dashboard metrics/events
- [ ] `D-020` Validate cross-device consistency for sessions and memory states
- [ ] `D-021` Add explicit offline-mode guarantees in app flow
- [ ] `D-022` Add operator runbook for sync incidents and release rollback

## Exit Criteria

1. Artifact release lifecycle is fully manageable from backend.
2. Mobile can bootstrap and update artifacts safely with offline continuity.
3. User progress backup/sync is reliable under conflict and connectivity issues.
