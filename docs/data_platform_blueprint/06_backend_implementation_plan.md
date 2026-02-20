# 06 - Backend Implementation Plan

Repo: `../iqrah-backend`

## Phase 1 - Extend Domain And Schema

1. Add release domain types in `crates/domain/src/`.
2. Add migrations for:
- `dataset_releases`
- `dataset_release_artifacts`
3. Add repository trait + Postgres impl:
- `ReleaseRepository`
- methods for create, attach artifact, validate, publish, list manifests.

## Phase 2 - Add API Handlers And Routes

1. New handlers in `crates/api/src/handlers/`:
- `admin_releases.rs`
- `releases.rs`
2. Register routes in `crates/api/src/routes/`.
3. Register schemas/paths in `crates/api/src/openapi.rs`.
4. Regenerate and commit `openapi.json`.

## Phase 3 - Validation Service

1. Add `ReleaseValidationService` (trait + impl) that checks:
- required artifact roles,
- pack publication status,
- storage file existence,
- metadata completeness.
2. Unit test with mocks.
3. Integration test for publish success/failure cases.

## Phase 4 - Admin Auth And Audit

1. Reuse admin API key middleware for release endpoints.
2. Log admin actions with actor identity + timestamp.
3. Add conflict-safe publish logic (single active latest release invariant).

## Phase 5 - Hardening

1. Add pagination for release listing.
2. Add idempotent re-validation.
3. Add rollback/deprecate endpoint behavior.

## Test Requirements Per Phase

1. Handler unit tests:
- happy path,
- validation error,
- unauthorized,
- downstream storage failure.

2. Repository integration tests:
- migration correctness,
- publish transaction atomicity,
- concurrent publish contention handling.

3. Contract checks:
- `just spec-check`,
- JSON schema shape stability for mobile client.
