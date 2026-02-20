# 12 - Backend Tickets (`../iqrah-backend`)

Reference docs:
1. `docs/data_platform_blueprint/04_release_model_and_api_contracts.md`
2. `docs/data_platform_blueprint/06_backend_implementation_plan.md`

## Phase 1 - Release Registry Core

- [ ] `B-001` Add release schema migrations
  - Deliverables:
    - migration for `dataset_releases`
    - migration for `dataset_release_artifacts`
    - indexes + constraints
  - Depends on: none
  - Acceptance:
    - migrations apply cleanly from empty DB
    - down/up migration path validated (if supported)

- [ ] `B-002` Add domain models and DTOs
  - Deliverables:
    - release entity types
    - request/response DTOs for admin/public release APIs
  - Depends on: `B-001`
  - Acceptance:
    - serde + schema tests pass
    - OpenAPI schemas compile

- [ ] `B-003` Add `ReleaseRepository` trait + Postgres implementation
  - Deliverables:
    - trait in storage layer
    - typed SQLx repository implementation
    - tests for create/attach/list/get
  - Depends on: `B-001`, `B-002`
  - Acceptance:
    - repository tests pass
    - no SQL in handlers

- [ ] `B-004` Add admin release handlers/routes
  - Endpoints:
    - `POST /v1/admin/releases`
    - `POST /v1/admin/releases/{id}/artifacts`
  - Depends on: `B-003`
  - Acceptance:
    - handler unit tests: happy/auth fail/validation fail/repo fail

- [ ] `B-005` Add release validation handler/logic
  - Endpoint:
    - `POST /v1/admin/releases/{id}/validate`
  - Rules:
    - required roles present
    - attached packages published
    - checksums/file metadata present
  - Depends on: `B-003`
  - Acceptance:
    - returns structured failures/warnings
    - validation tests cover failing and passing cases

- [ ] `B-006` Add release publish handler/logic
  - Endpoint:
    - `POST /v1/admin/releases/{id}/publish`
  - Depends on: `B-005`
  - Acceptance:
    - publish blocked when validation fails
    - transaction-safe status transition

- [ ] `B-007` Add public release endpoints
  - Endpoints:
    - `GET /v1/releases/latest`
    - `GET /v1/releases/{id}/manifest`
  - Depends on: `B-006`
  - Acceptance:
    - manifest includes required artifacts with package metadata needed by mobile bootstrap

- [ ] `B-008` OpenAPI integration
  - Deliverables:
    - `openapi.rs` updated
    - `openapi.json` regenerated and committed
  - Depends on: `B-004`, `B-005`, `B-006`, `B-007`
  - Acceptance:
    - `just spec`
    - `just spec-check`

## Phase 2 - Hardening and Operations

- [ ] `B-009` Add publish/deprecate audit logging
  - Depends on: `B-006`
  - Acceptance:
    - admin actions are persisted and queryable

- [ ] `B-010` Add release deprecate endpoint
  - Endpoint:
    - `POST /v1/admin/releases/{id}/deprecate`
  - Depends on: `B-006`
  - Acceptance:
    - deprecated release excluded from `latest`

- [ ] `B-011` Add compatibility/min-app-version checks
  - Depends on: `B-007`
  - Acceptance:
    - latest release can be filtered by client app version

- [ ] `B-012` Add pagination for admin release lists
  - Depends on: `B-006`
  - Acceptance:
    - deterministic paging and response metadata

- [ ] `B-013` Add admin CLI (release lifecycle)
  - Commands:
    - create, attach, validate, publish, deprecate, latest
  - Depends on: `B-008`, `B-010`
  - Acceptance:
    - CLI can perform full lifecycle on local backend

- [ ] `B-014` Add integration smoke workflow
  - Depends on: `B-013`
  - Acceptance:
    - CI smoke test runs full release lifecycle against test backend

## Phase 3 - Cleanup Support

- [ ] `B-015` Add release operation runbook docs in backend repo
  - Depends on: `B-014`
  - Acceptance:
    - operator steps for publish/rollback documented

## Backend Validation Commands (Required Per Ticket)

Run in `../iqrah-backend`:
1. `just fmt-check`
2. `just lint`
3. `just test`
4. `just coverage-ci`
5. `just spec-check`
6. `just sqlx-check` (when DB env is available)
