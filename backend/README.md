# Backend Development

This backend can be fully validated without running Flutter/frontend code.

## Local backend-only test commands

Run all backend checks from the `backend/` directory:

- `cargo fmt --all -- --check`
- `cargo clippy --all -- -D warnings`
- `cargo test --all`

Run crate-specific test targets:

- `cargo test -p iqrah-backend-storage --tests`
- `cargo test -p iqrah-backend-api --tests`


SQLite-first integration tests (no external services):

- `cargo test -p iqrah-backend-storage --test integration_sqlite_tests`

Each test creates an isolated temporary SQLite DB, runs migrations automatically, enables `PRAGMA foreign_keys=ON`, and cleans up DB files on success/failure.
Set `TEST_KEEP_DB=1` to keep DB files for debugging (path is printed in failure output).

Future Postgres-only suites are reserved under `crates/storage/tests/integration_postgres/` and should be gated behind `RUN_PG_TESTS=1` when introduced.


To run future/optional Postgres suites when available:

- `RUN_PG_TESTS=1 cargo test -p iqrah-backend-storage --tests --features postgres-tests`
- `RUN_PG_TESTS=1 cargo test -p iqrah-backend-api --tests --features postgres-tests`

(`RUN_PG_TESTS` is a human gate; `--features postgres-tests` is what enables those test targets.)

## Coverage (backend crates)

Install once:

- `cargo install cargo-llvm-cov`

Generate and enforce backend line coverage (85% minimum):

- `cargo llvm-cov --workspace --lcov --output-path target/llvm-cov/lcov.info --summary-only --fail-under-lines 85`
