# AGENTS.md

Canonical guidance for AI coding agents. Overrides `CLAUDE.md` and `.github/copilot-instructions.md` on conflict.

---

## 1. Specification Routing (READ FIRST)

| Domain | Task | File |
|:-------|:-----|:-----|
| **Backend** | New to backend | `backend/README.md` |
| | Domain types & errors | `backend/crates/domain/src/` |
| | DB schema | `backend/migrations/` |
| | Sync conflict logic | `backend/crates/storage/src/sync_repository.rs` |
| | Config & env vars | `backend/crates/config/src/lib.rs` |
| **Flutter / Rust core** | Architecture & data flow | `.github/copilot-instructions.md` |
| | Rust functions exposed to Flutter | `rust/src/api/` |

---

## 2. Build & Test

### Backend (run from `backend/`)

All three must pass before every push:

```bash
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test --all
```

SQLite integration tests (no external services):

```bash
cargo test -p iqrah-backend-storage --test integration_sqlite_tests
```

### Flutter / Rust core (run from repo root)

```bash
flutter test
flutter test integration_test
```

Regenerate bridge after any change to `rust/src/api/`:

```bash
flutter_rust_bridge_codegen generate
```

---

## 3. Locked Decisions (Non-Negotiable)

### Backend (Rust / Axum)

| # | Constraint |
|---|------------|
| 1 | `DomainError` is the only error type returned from handlers. Raw `sqlx::Error` or `anyhow::Error` in handler return types is **BANNED**. |
| 2 | Handlers never call `sqlx` directly. All DB access goes through repository functions. |
| 3 | String-concatenated SQL is **BANNED**. Use `query!` / `query_as!` macros only. |
| 4 | No hardcoded secrets, URLs, or credentials. All config comes from environment via `iqrah-backend-config`. |
| 5 | `println!` / `eprintln!` in backend code is **BANNED**. Use `tracing` macros. |
| 6 | `#[allow(dead_code)]` requires an inline comment explaining why. Never suppress silently. |
| 7 | `unwrap()` / `expect()` in non-test code is **BANNED**. Propagate with `?` or convert to `DomainError`. |
| 8 | `Arc<Mutex<T>>`, `Arc<RwLock<T>>`, and `DashMap` for shared mutable state are **BANNED**. |
| 9 | All shared mutable state must use an actor. Use `ractor` when the caller needs a response (RPC via `call!`); use a plain `tokio::mpsc` task for fire-and-forget. Handlers hold a cloneable `ActorRef<A>` or `Sender<Msg>` in `AppState`. All actors live under `backend/crates/api/src/actors/`. See `docs/backend/CONCURRENCY.md` for the full pattern and rework spec. |

### Flutter / Dart

| # | Constraint |
|---|------------|
| 1 | All state management via `flutter_riverpod`. No `setState` outside of trivial local UI state. |
| 2 | All business logic lives in Rust. Dart/Flutter is UI and state management only. |
| 3 | Never call `rust/src/api/` functions directly from widgets. Go through a Riverpod provider. |

---

## 4. Testing (Backend)

**85 % line coverage is a hard gate.** CI fails below this threshold. Do not merge code that drops coverage.

### Trait pattern (mandatory for all external dependencies)

Every component with an external dependency (DB, HTTP client, external API) must be behind a Rust trait so it can be tested with a fake in-process implementation — no live infrastructure required.

Follow the existing model in `backend/crates/api/src/handlers/auth.rs`:
- Define a trait for the dependency (`IdTokenVerifier`)
- Provide the real implementation (`GoogleIdTokenVerifier`)
- Place `#[cfg(test)]` fake implementations in the same module (`FakeVerifier`)

New repositories and external service calls must follow this pattern. Do **not** add handler logic that can only be exercised through a live DB or network call.

### Test pyramid

| Layer | Tool | Requirement |
|-------|------|-------------|
| Unit | `#[tokio::test]` + trait fakes | Test all handler logic in isolation |
| Storage integration | `#[sqlx::test]` with SQLite | SQLite-first; no external services |
| API integration | `axum::serve` in-process | Gate behind `--features postgres-tests` if Postgres is needed |

### SQLite integration tests

Each test gets an isolated temporary DB — migrations run automatically. Keep them the default integration path:

```bash
cargo test -p iqrah-backend-storage --test integration_sqlite_tests
TEST_KEEP_DB=1 cargo test ...   # keep DB file for debugging
```

---

## 5. Git Commit Style

Format: `type(scope?): subject`
Types: `feat`, `fix`, `perf`, `refactor`, `chore`, `build`, `test`, `docs`, `ci`
