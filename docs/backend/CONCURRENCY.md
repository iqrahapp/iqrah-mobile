# Backend Concurrency Rework

> Audited: 2026-02-19  
> Scope: `backend/` workspace

## Executive Summary

The backend has one mutable shared cache in request paths: pack integrity verification (`version_id -> verified`).

The primary issue is **naked ownership** (`Arc<DashMap<...>>` in `AppState` with reads/writes spread across files), not raw throughput. DashMap performance is acceptable for this use case.

**Decision:** introduce a typed wrapper (`PackVerificationCache`) and route all cache operations through it. Do not expose raw concurrent primitives in handlers or `AppState` APIs.

---

## Design Decision: Wrapper Struct vs Actor

| Situation | Use wrapper struct | Use actor |
|---|---:|---:|
| Operations are independent (`get/insert/remove/clear`) | ✅ | |
| Need atomic multi-step coordination across operations | | ✅ |
| Need mailbox semantics / backpressure / supervision | | ✅ |
| State can remain process-local utility cache | ✅ | |

For pack verification cache **today**, operations are independent and idempotent, so a wrapper is the simplest correct design.

---

## Pack Verification Cache Spec

File: `backend/crates/api/src/cache/pack_verification_cache.rs`

```rust
#[derive(Debug, Clone, Default)]
pub struct PackVerificationCache { ... }

impl PackVerificationCache {
    pub fn new() -> Self;
    pub fn is_verified(&self, version_id: i32) -> bool;
    pub fn mark_verified(&self, version_id: i32);
    pub fn invalidate(&self, version_id: i32);
    pub fn clear(&self);
}
```

Rules:
- Internal primitive can be `DashMap`, but it is private.
- `AppState` stores `PackVerificationCache`, never `Arc<DashMap<...>>`.
- Handlers call typed cache methods only.

---

## What Must Not Change

- Repositories remain stateless `PgPool` wrappers.
- `Arc<dyn IdTokenVerifier>` remains immutable shared dependency.
- `spawn_blocking` for Google JWT verification stays.
- Transaction logic in `SyncRepository` stays.

---

## Execution Plan (ordered)

1. Add `cache` module and `PackVerificationCache` wrapper.
2. Replace `AppState` raw cache field with `PackVerificationCache`.
3. Update startup wiring in `main.rs` to construct wrapper.
4. Refactor `packs.rs` to use wrapper methods and update tests.
5. Run checks:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all -- -D warnings`
   - `cargo test --all`

### Parallel-safe work

- Steps 1 and test fixture prep in step 4 can be developed in parallel.
- Steps 2 and 3 are tightly coupled and should stay sequential.
- Step 5 is final only.

---

## Future Actor Candidates

Use an actor when cross-operation coordination is required, e.g.:
- global rate limiting counters,
- in-flight download de-duplication,
- nonce/session invalidation workflows.
