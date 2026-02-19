# Backend Concurrency Rework

> Audited: 2026-02-19
> Scope: `backend/` workspace
> Input for: AI implementation scheduling agent

---

## Executive Summary

The backend has **one shared mutable primitive** that uses the wrong model:
`Arc<DashMap<i32, bool>>` in `AppState` (the pack integrity verification cache).

Everything else — repositories, pool, trait objects, DB transactions — is correctly designed and must not be changed.

**Verdict:** Replace `Arc<DashMap>` with a `PackCacheActor` using `kameo`. This eliminates internal shard locks, centralises cache ownership, and establishes the pattern all future shared state must follow.

---

## Current Shared State Inventory

| Primitive | Location | Verdict |
|-----------|----------|---------|
| `Arc<DashMap<i32, bool>>` (`verified_packs`) | `lib.rs:38`, `packs.rs:180,209,211,239`, `main.rs:49` | **REPLACE** with kameo actor |
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

Steps 1–3 are not atomic. If N concurrent requests arrive for the same version_id on a cold cache, all N pass the `contains_key` check and all N compute the SHA256. An actor serialises the check-and-insert, so only the first request computes the hash — all others wait for the actor's `ask` response and get `true` on their second call.

**4. Wrong precedent**

DashMap for the pack cache today means developers reach for `Arc<DashMap>` (or worse, `Arc<Mutex<HashMap>>`) when they need the next piece of shared state (rate limit counters, in-flight download tracking, session nonces). The codebase gradually accumulates concurrent primitives with no coherent ownership story. The actor pattern prevents this.

---

## Actor Pattern

### Rule

> A single Tokio task owns the state. Everything else communicates with it by sending messages. No shared reference to the state exists outside the task.

### Tool: kameo

`kameo` is a Tokio-native actor framework (v0.19, MSRV 1.88). It uses native async traits (no `async_trait` macro), per-message structs with associated reply types, and a clean `ask`/`tell` API.

| Operation | Method | Use when |
|-----------|--------|----------|
| RPC — caller needs a return value | `actor_ref.ask(msg).await` | Query operations |
| Fire-and-forget — no reply needed | `actor_ref.tell(msg).await` | Mutations, invalidations |

`ActorRef<A>` is `Clone` — store one in `AppState`, clone cheaply per handler.

### kameo API recap

```rust
use kameo::Actor;
use kameo::message::{Context, Message};

// 1. Actor struct — owns its state directly
#[derive(Actor)]
struct MyActor {
    cache: HashMap<i32, bool>,
}

// 2. One message struct per operation
struct Query(i32);   // RPC
struct Insert(i32);  // fire-and-forget

// 3. Implement Message<M> for each
impl Message<Query> for MyActor {
    type Reply = bool;
    async fn handle(&mut self, msg: Query, _: &mut Context<Self, Self::Reply>) -> bool {
        self.cache.contains_key(&msg.0)
    }
}

impl Message<Insert> for MyActor {
    type Reply = ();
    async fn handle(&mut self, msg: Insert, _: &mut Context<Self, Self::Reply>) {
        self.cache.insert(msg.0, true);
    }
}

// 4. Spawn — returns ActorRef<A> directly (not a tuple)
let actor_ref = MyActor::spawn(MyActor { cache: HashMap::new() });

// 5. Usage from handlers (ActorRef is Clone)
let found: bool = actor_ref.ask(Query(42)).await?;     // RPC
actor_ref.tell(Insert(42)).await?;                      // fire-and-forget
```

All actors live under `backend/crates/api/src/actors/`.

---

## Rework Spec: `PackCacheActor`

### New dependency

Add to `backend/crates/api/Cargo.toml`:

```toml
kameo = "0.19"
```

### New file: `backend/crates/api/src/actors/pack_cache.rs`

```rust
use std::collections::HashMap;
use kameo::Actor;
use kameo::message::{Context, Message};

#[derive(Actor)]
pub struct PackCacheActor {
    cache: HashMap<i32, bool>,
}

impl PackCacheActor {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }
}

/// RPC: returns true if version_id is already verified.
pub struct Query(pub i32);

/// Mark version_id as verified.
pub struct Insert(pub i32);

/// Remove a single version from the cache (on pack version invalidation).
pub struct Invalidate(pub i32);

/// Drop all entries (called when a new pack version is registered).
pub struct Clear;

impl Message<Query> for PackCacheActor {
    type Reply = bool;
    async fn handle(&mut self, msg: Query, _: &mut Context<Self, Self::Reply>) -> bool {
        self.cache.contains_key(&msg.0)
    }
}

impl Message<Insert> for PackCacheActor {
    type Reply = ();
    async fn handle(&mut self, msg: Insert, _: &mut Context<Self, Self::Reply>) {
        self.cache.insert(msg.0, true);
    }
}

impl Message<Invalidate> for PackCacheActor {
    type Reply = ();
    async fn handle(&mut self, msg: Invalidate, _: &mut Context<Self, Self::Reply>) {
        self.cache.remove(&msg.0);
    }
}

impl Message<Clear> for PackCacheActor {
    type Reply = ();
    async fn handle(&mut self, _: Clear, _: &mut Context<Self, Self::Reply>) {
        self.cache.clear();
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
pub pack_cache: ActorRef<PackCacheActor>,
```

**`invalidate_pack_cache`** — replace:

```rust
// Before
self.verified_packs.remove(&pack_version_id);

// After
let _ = self.pack_cache.tell(Invalidate(pack_version_id)).await;
```

**`add_pack_version`** — replace:

```rust
// Before
self.verified_packs.clear();

// After
let _ = self.pack_cache.tell(Clear).await;
```

Remove `use dashmap::DashMap;` import.

### Change: `backend/crates/api/src/main.rs`

**Spawn actor before building AppState**:

```rust
// Before
verified_packs: Arc::new(DashMap::new()),

// After — spawn returns ActorRef<PackCacheActor> directly
let pack_cache = PackCacheActor::spawn(PackCacheActor::new());
// ...
pack_cache,
```

Remove `use dashmap::DashMap;` import.

### Change: `backend/crates/api/src/handlers/packs.rs`

**`verify_pack_integrity` signature** — replace `&DashMap<i32, bool>` parameter with `ActorRef<PackCacheActor>`.

**Cache read** — replace:

```rust
// Before
if verified_packs.contains_key(&version_id) {
    return Ok(());
}
// ...
verified_packs.insert(version_id, true);

// After
let is_cached = pack_cache
    .ask(Query(version_id))
    .await
    .map_err(|e| DomainError::Internal(anyhow::anyhow!("pack cache: {}", e)))?;

if is_cached {
    return Ok(());
}
// ...
let _ = pack_cache.tell(Insert(version_id)).await;
```

Remove `use dashmap::DashMap;` import.

### Error handling note

`ask` returns `Result<Reply, SendError<M, E>>`. Map this to `DomainError::Internal` inline in the `api` crate — do not add a `From` impl in `domain` as that would couple the domain crate to kameo.

`tell` returns `Result<(), SendError<M>>`. For fire-and-forget cache mutations (Insert, Invalidate, Clear), swallowing the error with `let _ = ...` is acceptable — if the actor has crashed the request can still succeed (the next download will re-verify).

### `AppState` Clone

`ActorRef<PackCacheActor>` implements `Clone`, so `#[derive(Clone)]` on `AppState` continues to work with no changes.

### Change: `backend/Cargo.toml`

Remove `dashmap` if it was declared (it was pulled as a transitive dep — verify after removing all usages).

---

## Pack Cache Actor: Deduplication Limitation

The design above replaces `DashMap` with an actor that caches SHA256 *results*. It does not deduplicate concurrent *computations*. Under N concurrent first-requests for the same cold version_id, all N will:

1. `ask(Query(id))` → `false`
2. Start computing SHA256 independently
3. `tell(Insert(id))` when done (idempotent, all N writes are no-ops after the first)

This is the same race as with DashMap — the actor just owns the state more cleanly.

**If deduplication of work matters** (large pack files, heavy SHA256 load), the actor can be extended to track in-flight computations. The actor would hold a `HashMap<i32, Vec<oneshot::Sender<bool>>>`: on a `Query` for an in-flight version_id, the actor parks the reply sender and resolves all of them when `Insert` arrives. This is a future improvement, not a blocker for the initial rework.

---

## Future Actor Candidates

The following features are currently absent from the backend (documented in `docs/backend/AUDIT.md`). When they are implemented, they **must** use actors for any shared mutable state — do not reach for `Mutex` or `DashMap`.

| Feature | Why an actor | When triggered |
|---------|-------------|----------------|
| Rate limiting | Per-IP or per-user request counters are shared mutable state | When `tower_governor` (already in deps) is wired up, or a custom rate limiter is built |
| JWT token cache | Caching verified tokens to avoid re-running `decode` on every request is shared mutable state | If JWT verification becomes a latency bottleneck |
| In-flight download tracking | Tracking active streams to enforce per-user concurrency limits or cancel on disconnect | If download concurrency control is needed |

---

## What Must Not Change

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

1. Does a handler need a response from the operation? → use `actor_ref.ask(msg).await`
2. Is it fire-and-forget? → use `actor_ref.tell(msg).await`
3. Is the state truly read-only after init? → `Arc<T>` or a plain field is fine

`Arc<Mutex<T>>`, `Arc<RwLock<T>>`, and `Arc<DashMap<K,V>>` for mutable state are never acceptable.

---

## Implementation Tasks

For the scheduling agent — execute in order, each is independently verifiable:

**Task 1 — Add kameo dependency**
- File: `backend/crates/api/Cargo.toml`
- Add `kameo = "0.19"` under `[dependencies]`
- Run `cargo build -p iqrah-backend-api` to confirm it resolves

**Task 2 — Create `PackCacheActor`**
- Create `backend/crates/api/src/actors/mod.rs`
- Create `backend/crates/api/src/actors/pack_cache.rs` with the spec above
- Add `mod actors;` to `backend/crates/api/src/lib.rs`
- Run `cargo build -p iqrah-backend-api` to confirm it compiles

**Task 3 — Update `AppState`**
- File: `backend/crates/api/src/lib.rs`
- Replace `verified_packs: Arc<DashMap<i32, bool>>` with `pack_cache: ActorRef<PackCacheActor>`
- Update `invalidate_pack_cache` and `add_pack_version` to use `tell`
- Remove `use dashmap::DashMap`

**Task 4 — Update `main.rs`**
- File: `backend/crates/api/src/main.rs`
- Spawn `PackCacheActor` before constructing `AppState`
- Pass `pack_cache` handle into `AppState`
- Remove `use dashmap::DashMap`

**Task 5 — Update `packs.rs`**
- File: `backend/crates/api/src/handlers/packs.rs`
- Change `verify_pack_integrity` signature: replace `&DashMap<i32, bool>` with `ActorRef<PackCacheActor>`
- Replace `contains_key` + `insert` with `ask(Query(...))` + `tell(Insert(...))`
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
