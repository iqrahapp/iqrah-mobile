# 13 - Mobile Tickets (`iqrah-mobile`)

Reference docs:
1. `docs/data_platform_blueprint/02_testability_strategy.md`
2. `docs/data_platform_blueprint/05_mobile_runtime_changes.md`
3. `docs/data_platform_blueprint/07_migration_and_cutover.md`

## Phase 1 - Testability Foundation

- [ ] `M-001` Add deterministic golden session scenarios
  - Deliverables:
    - cold-start scenario
    - due-review scenario
    - chunk-focused scenario
  - Depends on: none
  - Acceptance:
    - scenario output snapshot tests fail on behavioral drift

- [ ] `M-002` Add invariant tests for session/exercise pipeline
  - Invariants:
    - no duplicate node ids
    - non-empty cold start
    - limit respected
    - valid node/axis/exercise mapping
  - Depends on: `M-001`
  - Acceptance:
    - tests run in CI with deterministic fixtures

## Phase 2 - Bootstrap Infrastructure (Feature Flag)

- [ ] `M-003` Add release manifest client models/services
  - Depends on: `B-007`, `B-008`
  - Acceptance:
    - client can fetch `/v1/releases/latest` and parse manifest

- [ ] `M-004` Add `ArtifactDownloader` with resume support
  - Depends on: `M-003`
  - Acceptance:
    - interrupted download resumes
    - checksum verification utility implemented

- [ ] `M-005` Add `LocalReleaseRegistry`
  - Tracks:
    - active release id/version
    - installed artifacts and checksums
  - Depends on: `M-003`
  - Acceptance:
    - registry persists and restores across app restarts

- [ ] `M-006` Add `AtomicActivationManager`
  - Depends on: `M-004`, `M-005`
  - Acceptance:
    - activation swap is atomic
    - failed activation leaves previous release active

- [ ] `M-007` Add feature-flagged bootstrap path in startup
  - Depends on: `M-006`
  - Acceptance:
    - flag on: bootstrap from backend
    - flag off: existing local bundled path still works

## Phase 3 - Default Remote + Hardening

- [ ] `M-008` Make remote bootstrap default for non-dev builds
  - Depends on: `B-011`, `M-007`
  - Acceptance:
    - fresh install succeeds without bundled heavy artifact dependency

- [ ] `M-009` Add bootstrap telemetry and health reporting
  - Depends on: `M-008`
  - Acceptance:
    - emit fetch/download/checksum/activation metrics

- [ ] `M-010` Add rollback-on-failure and safe retry logic
  - Depends on: `M-006`, `M-009`
  - Acceptance:
    - bad update never bricks app startup

- [ ] `M-011` Add end-to-end integration tests for bootstrap/install/update
  - Depends on: `M-010`
  - Acceptance:
    - automated tests cover first install, interrupted download, update activation

- [ ] `M-012` Add offline guarantee tests
  - Depends on: `M-011`
  - Acceptance:
    - app can run learning flow offline after successful install

- [ ] `M-013` Remove hidden coupling to bundled `rust/content.db` in default path
  - Depends on: `M-008`, `M-012`
  - Acceptance:
    - default runtime path uses active installed release artifacts only

## Phase 4 - Repo Cleanup

- [ ] `M-014` Remove heavy generated artifacts from default repo workflow
  - Depends on: `M-013`, `B-015`
  - Acceptance:
    - heavy artifacts no longer required for normal app startup

- [ ] `M-015` Update dev onboarding docs and scripts
  - Depends on: `M-014`
  - Acceptance:
    - clear setup for local fallback and fixture-only test flows

## Mobile Validation Commands (Required Per Ticket)

Run from `iqrah-mobile` root:
1. `flutter test`
2. `flutter test integration_test`
3. Any Rust tests added for `rust/crates/iqrah-core` and `rust/crates/iqrah-api`
