# 04 - Ticket Board: Data Platform, Backend, Sync

Source anchors:
1. `docs/data_platform_blueprint/12_tickets_backend.md`
2. `docs/data_platform_blueprint/13_tickets_mobile.md`
3. `docs/ai_blueprint-3/06_backend_scope_and_sync_contract.md`
4. `docs/ai_blueprint-3/09_claims_vs_runtime_reality.md`

## Data-Blueprint Verification Gate

- [ ] `[BOTH]` `D-000` Audit and score `docs/data_platform_blueprint` against current code/runtime
  - Deliverables:
    - short scorecard (accuracy, completeness, actionability),
    - corrections list,
    - ticket updates before `D-001` starts if needed.
  - Accept:
    - no `D` implementation proceeds on unverified assumptions.

## Backend Release Registry

- [ ] `[BE]` `D-001` Add release schema migrations (`dataset_releases`, `dataset_release_artifacts`)
- [ ] `[BE]` `D-002` Add release domain models and DTOs
- [ ] `[BE]` `D-003` Add `ReleaseRepository` + tests
- [ ] `[BE]` `D-004` Add admin release create/attach endpoints
- [ ] `[BE]` `D-005` Add release validation endpoint + rule engine
- [ ] `[BE]` `D-006` Add release publish endpoint (validation-gated)
- [ ] `[BE]` `D-007` Add public release endpoints (`latest`, `manifest`)
- [ ] `[BE]` `D-008` Add OpenAPI coverage + regenerate spec
- [ ] `[BE]` `D-009` Add release deprecate endpoint
- [ ] `[BE]` `D-010` Add admin audit logging for release actions

## Mobile Bootstrap Integration

- [ ] `[MOB]` `D-011` Add release manifest client/service in mobile
- [ ] `[MOB]` `D-012` Add resumable downloader with checksum verification
- [ ] `[MOB]` `D-013` Add local release registry persistence
- [ ] `[MOB]` `D-014` Add atomic activation manager
- [ ] `[MOB]` `D-015` Add feature-flagged remote bootstrap startup path
- [ ] `[MOB]` `D-016` Make remote bootstrap default for non-dev builds
- [ ] `[MOB]` `D-017` Add rollback + retry safety for failed updates

## Sync/Backup Hardening

- [ ] `[BOTH]` `D-018` Add two-device conflict integration tests
- [ ] `[BOTH]` `D-019` Add sync observability dashboard metrics/events
- [ ] `[BOTH]` `D-020` Validate cross-device consistency for sessions and memory states
- [ ] `[MOB]` `D-021` Add explicit offline-mode guarantees in app flow
- [ ] `[BOTH]` `D-022` Add operator runbook for sync incidents and release rollback

## Exit Criteria

1. Artifact release lifecycle is fully manageable from backend.
2. Mobile can bootstrap and update artifacts safely with offline continuity.
3. User progress backup/sync is reliable under conflict and connectivity issues.
