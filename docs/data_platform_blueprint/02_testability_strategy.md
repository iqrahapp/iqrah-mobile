# 02 - Testability Strategy

Goal: remove dependence on manual regression checks for core learning flow.

## Test Pyramid

1. Domain/unit tests (Rust/Dart)
- Pure logic and mapping correctness.
- No DB/network required.

2. Repository/service integration tests (Rust)
- SQLite-backed tests for mobile core data access.
- Postgres-backed tests for backend repository behavior.

3. API/bridge contract tests
- FRB contract tests for mobile Rust APIs.
- Backend HTTP contract tests using generated OpenAPI.

4. End-to-end scenario tests
- Deterministic session flow tests from setup to summary.
- Artifact bootstrap/install/update tests.

## Mandatory Golden Scenarios

1. Cold-start user gets non-empty session.
2. Due-review user receives valid due queue with stable ordering properties.
3. Chunk-focused session includes continuity + due + lexical budgets.
4. Exercise generation mapping is valid for node/axis combinations.
5. Session completion persists memory/session/session-item writes.
6. Sync push/pull round-trip preserves state.
7. Artifact bootstrap installs and activates expected release.

## Invariant Tests (Must Always Pass)

1. No duplicate node IDs inside a session.
2. Session limit is respected.
3. Unsupported exercise types are not emitted in scheduled path.
4. Goal/chunk filter changes candidate pool when set.
5. Checksum mismatch prevents activation.
6. App can recover from interrupted download.

## CI Gates

For this repo:
1. `flutter test`
2. `flutter test integration_test`
3. Rust core tests for session/scheduler/services
4. Golden scenario diff check

For backend repo:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo nextest run --workspace`
4. `cargo llvm-cov --workspace --fail-under-lines 85`
5. `just spec-check`

## Test Data Discipline

1. Keep tiny deterministic fixtures under version control.
2. Do not use production-sized data in CI.
3. Keep a dedicated fixture generator script for reproducible snapshots.
4. Fail tests if fixture schema version is stale.

## Automation Requirement

Any change to:
1. scheduler/session logic,
2. exercise routing,
3. pack/release contracts,
4. bootstrap/install flow

must include at least one new automated test that would fail without that change.
