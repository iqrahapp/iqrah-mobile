# Backend Concurrency Rework

> Audited: 2026-02-19
> Scope: `backend/` workspace
> Input for: AI implementation scheduling agent

---

## Executive Summary

The backend has **one shared mutable primitive** that uses the wrong model:
`Arc<DashMap<i32, bool>>` in `AppState` (the pack integrity verification cache).

Everything else — repositories, pool, trait objects, DB transactions — is correctly designed and must not be changed.

**Verdict:** Replace `Arc<DashMap>` with a `PackCacheActor` using `ractor`. This eliminates internal shard locks, centralises cache ownership, and establishes the pattern all future shared state must follow.

---

## Current Shared State Inventory

| Primitive | Location | Verdict |
|-----------|----------|---------|
| `Arc<DashMap<i32, bool>>` (`verified_packs`) | `lib.rs:38`, `packs.rs:180,209,211,239`, `main.rs:49` | **REPLACE** with ractor actor |
| `Arc<dyn IdTokenVerifier>` | `lib.rs:37`, `auth.rs:138` | OK — shared immutable trait object |
| `Arc<AppState>` | every handler via Axum `State` extractor | OK — Axum standard, read-only after init |
| `spawn_blocking` (Google JWT verify) | `auth.rs:140` | OK — correct use for sync crypto call |
| `PgPool` (Arc-backed internally) | all repository structs | OK — pool owns all connection concurrency |
| Postgres transactions | `sync_repository.rs` (`pool.begin()`) | OK — database handles isolation |
| `Mutex` / `RwLock` | **nowhere** | N/A |

---

## Issue: `Arc<DashMap<i32, bool>>`

### What it does

`verified_packs` caches the result of SHA256 integrity checks. When a pack file is first downloaded, the server hashes the file on disk and compares it to the stored checksum. If it matches, `version_id → true` is written to the map. Subsequent downloads skip the hash.

Cache is invalidated (`.remove()`) when a pack version is invalidated, and fully cleared (`.clear()`) when a new version is added.

### Why DashMap is the wrong model

**1. Internal shard locks — not lock-free**

DashMap uses shard-based `parking_lot::RwLock` internally (one lock per shard, typically 16 shards). Under concurrent download traffic every `contains_key` and `insert` acquires a shard lock. It is faster than a single `Mutex<HashMap>` but it is not lock-free, and the lock is hidden — callers cannot reason about contention.

**2. Ownership is spread across three files**

| Operation | Location |
|-----------|----------|
| Initialisation (`DashMap::new()`) | `main.rs:49` |
| Read (`contains_key`) | `packs.rs:211` |
| Write (`insert`) | `packs.rs:239` |
| Invalidate (`remove`) | `lib.rs:45` (`invalidate_pack_cache`) |
| Clear (`clear`) | `lib.rs:68` (`add_pack_version`) |

No single file owns the cache. Adding a new operation (e.g. a TTL eviction) requires touching multiple files and reasoning about concurrent access everywhere.

**3. Non-atomic check-then-act**

`verify_pack_integrity` (`packs.rs:204`) does:

```
1. contains_key(version_id)?  →  false (cache miss)
2. compute_pack_sha256(...)   →  expensive async I/O + CPU
3. insert(version_id, true)   →  write to map
```

Steps 1–3 are not atomic. If N concurrent requests arrive for the same version_id on a cold cache, all N pass the `contains_key` check and all N compute the SHA256. An actor serialises the check-and-insert, so only the first request computes the hash — all others wait for the actor's `Query` response and get `true` on their second call.

**4. Wrong precedent**

DashMap for the pack cache today means developers reach for `Arc<DashMap>` (or worse, `Arc<Mutex<HashMap>>`) when they need the next piece of shared state (rate limit counters, in-flight download tracking, session nonces). The codebase gradually accumulates concurrent primitives with no coherent ownership story. The actor pattern prevents this.

---

## Actor Pattern

### Rule

> A single Tokio task owns the state. Everything else communicates with it by sending messages. No shared reference to the state exists outside the task.

### When to use ractor vs custom mpsc

| Situation | Tool |
|-----------|------|
| Caller needs a return value (RPC) | `ractor` — `call!` macro handles the oneshot pairing |
| Fire-and-forget, no response needed | `tokio::mpsc` task or `ractor` with `cast!` |
| Actor needs supervision / restart | `ractor` |
| Simple, isolated, no RPC | plain `tokio::mpsc` task |

The pack cache needs RPC (`"is this version already verified?"` → `bool`), so `ractor` is the right choice.

### ractor API recap

```rust
use ractor::{Actor, ActorRef, ActorProcessingErr, RpcReplyPort};

// 1. Message enum
#[derive(Debug)]
pub enum MyMsg {
    Query(SomeKey, RpcReplyPort<bool>),   // RPC: caller awaits reply
    Insert(SomeKey),                       // fire-and-forget
}
impl ractor::Message for MyMsg {}

// 2. Actor impl
pub struct MyActor;

impl Actor for MyActor {
    type Msg   = MyMsg;
    type State = HashMap<SomeKey, bool>;
    type Arguments = ();

    async fn pre_start(
        &self, _myself: ActorRef<Self::Msg>, _args: ()
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(HashMap::new())
    }

    async fn handle(
        &self, _myself: ActorRef<Self::Msg>,
        msg: Self::Msg, state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            MyMsg::Query(key, reply) => {
                let _ = reply.send(state.contains_key(&key));
            }
            MyMsg::Insert(key) => { state.insert(key, true); }
        }
        Ok(())
    }
}

// 3. Spawn (once, at startup) — returns (ActorRef, JoinHandle)
let (actor_ref, _handle) = Actor::spawn(None, MyActor, ()).await?;

// 4. Usage from handlers (ActorRef is Clone)
let is_cached = call!(actor_ref, MyMsg::Query, key)?;   // RPC
cast!(actor_ref, MyMsg::Insert(key))?;                  // fire-and-forget
```

All actors live under `backend/crates/api/src/actors/`.

---

## Rework Spec: `PackCacheActor`

### New dependency

Add to `backend/crates/api/Cargo.toml`:

```toml
ractor = "0.15"
```

### New file: `backend/crates/api/src/actors/pack_cache.rs`

```rust
use std::collections::HashMap;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort};

#[derive(Debug)]
pub enum PackCacheMsg {
    /// RPC: returns true if version_id is already verified.
    Query(i32, RpcReplyPort<bool>),
    /// Mark version_id as verified.
    Insert(i32),
    /// Remove a single version from the cache (on pack version invalidation).
    Invalidate(i32),
    /// Drop all entries (called when a new pack version is registered).
    Clear,
}
impl ractor::Message for PackCacheMsg {}

pub struct PackCacheActor;

impl Actor for PackCacheActor {
    type Msg       = PackCacheMsg;
    type State     = HashMap<i32, bool>;
    type Arguments = ();

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _args: (),
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(HashMap::new())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match msg {
            PackCacheMsg::Query(version_id, reply) => {
                let _ = reply.send(state.contains_key(&version_id));
            }
            PackCacheMsg::Insert(version_id) => {
                state.insert(version_id, true);
            }
            PackCacheMsg::Invalidate(version_id) => {
                state.remove(&version_id);
            }
            PackCacheMsg::Clear => {
                state.clear();
            }
        }
        Ok(())
    }
}
```

New module file: `backend/crates/api/src/actors/mod.rs`

```rust
pub mod pack_cache;
```

### Change: `backend/crates/api/src/lib.rs`

**`AppState` field** — replace:

```rust
// Before
pub verified_packs: Arc<DashMap<i32, bool>>,

// After
pub pack_cache: ActorRef<PackCacheMsg>,
```

**`invalidate_pack_cache`** — replace:

```rust
// Before
self.verified_packs.remove(&pack_version_id);

// After
cast!(self.pack_cache, PackCacheMsg::Invalidate(pack_version_id))?;
```

**`add_pack_version`** — replace:

```rust
// Before
self.verified_packs.clear();

// After
cast!(self.pack_cache, PackCacheMsg::Clear)?;
```

Remove `use dashmap::DashMap;` import.

### Change: `backend/crates/api/src/main.rs`

**Spawn actor before building AppState**:

```rust
// Before
verified_packs: Arc::new(DashMap::new()),

// After — spawn returns (ActorRef, JoinHandle)
let (pack_cache, _handle) = Actor::spawn(None, PackCacheActor, ())
    .await
    .expect("failed to start PackCacheActor");
// ...
pack_cache,
```

Remove `use dashmap::DashMap;` import.

### Change: `backend/crates/api/src/handlers/packs.rs`

**`verify_pack_integrity` signature** — replace `&DashMap<i32, bool>` parameter with `ActorRef<PackCacheMsg>`.

**Cache read** — replace:

```rust
// Before
if verified_packs.contains_key(&version_id) {
    return Ok(());
}
// ...
verified_packs.insert(version_id, true);

// After
if call!(pack_cache, PackCacheMsg::Query, version_id)? {
    return Ok(());
}
// ...
cast!(pack_cache, PackCacheMsg::Insert(version_id))?;
```

Remove `use dashmap::DashMap;` import.

### Error mapping

`call!` and `cast!` return ractor-specific errors (`ractor::RactorErr`, `ractor::MessagingErr`). Handlers return `DomainError`. Add a `From` impl in `domain/src/errors.rs` or map inline:

```rust
// Option A: From impl (preferred — keeps handlers clean)
impl From<ractor::RactorErr<PackCacheMsg>> for DomainError {
    fn from(err: ractor::RactorErr<PackCacheMsg>) -> Self {
        DomainError::Internal(anyhow::anyhow!("actor error: {}", err))
    }
}

// Option B: inline .map_err (if you want to avoid coupling domain to ractor)
call!(pack_cache, PackCacheMsg::Query, version_id)
    .map_err(|e| DomainError::Internal(anyhow::anyhow!("pack cache actor: {}", e)))?;
```

Option B is recommended — it keeps the `domain` crate free of ractor dependency. The `map_err` lives in the `api` crate where ractor is already a dependency.

### `AppState` Clone

`ActorRef<PackCacheMsg>` implements `Clone`, so `#[derive(Clone)]` on `AppState` continues to work with no changes.

### Change: `backend/Cargo.toml`

Remove `dashmap` if it was declared (it was a transitive dep via AppState — check after removing all usages).

---

## What Must Not Change

These patterns are correct and must be preserved:

| Component | Why it is correct |
|-----------|-------------------|
| `PackRepository` / `UserRepository` / `SyncRepository` | Stateless `PgPool` wrappers. `PgPool` owns all connection concurrency internally. |
| `Arc<dyn IdTokenVerifier>` | Shared **immutable** trait object. No mutation after init. |
| `spawn_blocking` for Google JWT | The `google-jwt-verify` API is synchronous and must not block the async executor. |
| Postgres transactions in `SyncRepository` | Database transaction isolation handles concurrent sync pushes correctly. |
| `Arc<AppState>` in Axum `State` | Standard Axum pattern for read-only shared state. |

---

## Future Shared State Rule

Before adding any field to `AppState` that is mutated after startup, ask:

1. Does a handler need a response from the operation? → use `ractor` + `call!`
2. Is it fire-and-forget? → `ractor` with `cast!` or a plain `tokio::mpsc` task
3. Is the state truly read-only after init? → `Arc<T>` or a plain field is fine

`Arc<Mutex<T>>`, `Arc<RwLock<T>>`, and `Arc<DashMap<K,V>>` for mutable state are never acceptable.

---

## Implementation Tasks

For the scheduling agent — execute in order, each is independently verifiable:

**Task 1 — Add ractor dependency**
- File: `backend/crates/api/Cargo.toml`
- Add `ractor = "0.15"` under `[dependencies]`
- Run `cargo build -p iqrah-backend-api` to confirm it resolves

**Task 2 — Create `PackCacheActor`**
- Create `backend/crates/api/src/actors/mod.rs`
- Create `backend/crates/api/src/actors/pack_cache.rs` with the spec above
- Add `mod actors;` to `backend/crates/api/src/lib.rs`
- Run `cargo build -p iqrah-backend-api` to confirm it compiles

**Task 3 — Update `AppState`**
- File: `backend/crates/api/src/lib.rs`
- Replace `verified_packs: Arc<DashMap<i32, bool>>` with `pack_cache: ActorRef<PackCacheMsg>`
- Update `invalidate_pack_cache` and `add_pack_version` to use `cast!`
- Remove `use dashmap::DashMap`

**Task 4 — Update `main.rs`**
- File: `backend/crates/api/src/main.rs`
- Spawn `PackCacheActor` before constructing `AppState`
- Pass `pack_cache` handle into `AppState`
- Remove `use dashmap::DashMap`

**Task 5 — Update `packs.rs`**
- File: `backend/crates/api/src/handlers/packs.rs`
- Change `verify_pack_integrity` signature: replace `&DashMap<i32, bool>` with `ActorRef<PackCacheMsg>`
- Replace `contains_key` with `call!` and `insert` with `cast!`
- Remove `use dashmap::DashMap`
- Update call sites to pass `state.pack_cache.clone()`

**Task 6 — Run full CI checks**

```bash
cd backend
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
cargo test --all
```

All must pass before closing this rework.
