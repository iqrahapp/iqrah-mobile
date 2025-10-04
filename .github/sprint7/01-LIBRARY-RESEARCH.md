# Sprint 7: Library Research & Recommendations

**Date:** 2025-10-04
**Purpose:** Identify production-ready libraries to accelerate Sprint 7 refactoring

---

## Core Principle: Don't Reinvent the Wheel

We will leverage battle-tested Rust libraries for:
1. Database abstraction and query building
2. Migration management
3. Dependency injection
4. Testing utilities
5. CLI tooling

---

## 1. Database Layer & Query Building

### ⭐ **Recommended: SQLx**
**Crate:** `sqlx` (v0.7+)
**Why:**
- Compile-time checked SQL queries (prevents SQL injection, typos)
- Async/await native support (works with Tokio)
- Connection pooling built-in
- Supports transactions
- Zero-cost abstractions

**Example:**
```rust
use sqlx::{SqlitePool, query_as};

#[derive(sqlx::FromRow)]
struct NodeData {
    id: String,
    node_type: NodeType,
}

let nodes = query_as!(
    NodeData,
    "SELECT id, node_type FROM nodes WHERE id = ?",
    node_id
)
.fetch_all(&pool)
.await?;
```

**Benefits for Iqrah:**
- Eliminates manual `rusqlite` boilerplate
- Macros verify queries at compile time against schema
- Better error messages
- Prepared statements by default

**Alternative:** `diesel` (more ORM-like, but heavier and less async-friendly)

---

## 2. Migration Framework

### ⭐ **Recommended: sqlx-cli + embedded migrations**
**Crate:** `sqlx` (includes migration support)
**Why:**
- Built into SQLx
- Version-based migration files
- Up/down migration support
- Embeddable migrations (ship in binary)
- Simple CLI tool for development

**Example:**
```bash
# Create migration
sqlx migrate add create_user_stats

# Generated files:
# migrations/20251004_create_user_stats.sql
```

```rust
// Embed in binary
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

**Alternatives:**
- `refinery` - Migration framework (good, but less integrated)
- `barrel` - Schema builder (too complex for our needs)
- **Custom PRAGMA user_version** - We'll keep this as a lightweight backup

---

## 3. Dependency Injection / Service Container

### ⭐ **Recommended: Constructor Injection (Manual DI)**
**Why:**
- Rust's ownership system makes DI natural
- No magic, explicit dependencies
- Better for testing

**Pattern:**
```rust
pub struct LearningService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
    scheduler: Arc<dyn Scheduler>,
}

impl LearningService {
    pub fn new(
        content_repo: Arc<dyn ContentRepository>,
        user_repo: Arc<dyn UserRepository>,
        scheduler: Arc<dyn Scheduler>,
    ) -> Self {
        Self { content_repo, user_repo, scheduler }
    }
}
```

**Alternative (if needed):** `shaku` - Lightweight DI container
- Not recommended yet; manual DI is cleaner in Rust

---

## 4. Testing Utilities

### ⭐ **Recommended: Combination of Tools**

#### 4.1. **Mocking: `mockall`**
```rust
use mockall::*;

#[automock]
trait UserRepository {
    fn get_memory_state(&self, node_id: &str) -> Result<MemoryState>;
}

#[test]
fn test_scheduler() {
    let mut mock_repo = MockUserRepository::new();
    mock_repo.expect_get_memory_state()
        .returning(|_| Ok(MemoryState::default()));

    let service = LearningService::new(Arc::new(mock_repo));
    // Test...
}
```

#### 4.2. **Property Testing: `proptest`**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_energy_always_between_0_and_1(energy_delta in -1.0f64..1.0f64) {
        let result = apply_energy_change(0.5, energy_delta);
        prop_assert!(result >= 0.0 && result <= 1.0);
    }
}
```

#### 4.3. **Fixtures: `rstest`**
```rust
use rstest::*;

#[fixture]
fn db_pool() -> SqlitePool {
    // Create in-memory test DB
    SqlitePool::connect(":memory:").await.unwrap()
}

#[rstest]
fn test_with_db(db_pool: SqlitePool) {
    // Test uses fresh DB
}
```

#### 4.4. **Async Testing: `tokio::test`**
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_operation().await;
    assert_eq!(result, expected);
}
```

---

## 5. CLI Tooling

### ⭐ **Recommended: `clap` v4**
**Crate:** `clap` (derive API)
**Why:**
- Best CLI framework in Rust
- Derive macros for ergonomic API
- Auto-generated help
- Subcommands support

**Example:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "iqrah-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test scoring algorithm
    TestScoring {
        #[arg(short, long)]
        node_id: String,
    },
    /// Generate session preview
    Session {
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },
    /// Run migrations
    Migrate,
}
```

**Companion: `clap_complete`** - Shell completion generation

---

## 6. Error Handling

### ⭐ **Current: `anyhow`** (Keep it)
**Why:**
- Already in use
- Perfect for application-level errors
- Context chaining

**Complement with: `thiserror`** for library errors
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("No due items found for user {0}")]
    NoDueItems(String),

    #[error("Invalid energy value: {0}")]
    InvalidEnergy(f64),
}
```

---

## 7. Serialization (Keep Current)

### ⭐ **Current: `serde` + `serde_json`** ✅
**Why:**
- Industry standard
- Works perfectly with FRB
- No changes needed

---

## 8. Observability & Logging

### ⭐ **Recommended: `tracing`**
**Crate:** `tracing` + `tracing-subscriber`
**Why:**
- Structured logging
- Spans for context
- Better than `println!`
- Integrates with async

**Example:**
```rust
use tracing::{info, debug, error, instrument};

#[instrument]
async fn process_review(node_id: &str, grade: ReviewGrade) -> Result<()> {
    debug!("Processing review for node {}", node_id);
    // ...
    info!(new_energy = ?state.energy, "Review processed");
    Ok(())
}
```

---

## 9. Database Connection Pooling

### ⭐ **SQLx Built-in** (already using `r2d2` - migrate to SQLx)
**Current:** `r2d2` + `rusqlite`
**Recommended:** `sqlx::SqlitePool`

**Why:**
- Native async support
- Better integration with SQLx
- Simpler API

---

## 10. Workflow / State Machines (Future)

### 📋 **Consider Later: `state_machine_future`**
**For:** Exercise difficulty progression
**Why:**
- Type-safe state transitions
- Compile-time guarantees
- Perfect for exercise variant selection logic

**Not urgent** - Implement in Sprint 8+

---

## Library Dependency Tree (Sprint 7)

```toml
[dependencies]
# Database & Queries
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite", "macros"] }

# Error Handling
anyhow = "1.0"  # Keep
thiserror = "1.0"  # Add

# Async Runtime (existing)
tokio = { version = "1", features = ["full"] }

# CLI (new workspace member)
clap = { version = "4", features = ["derive"] }

# Testing (dev-dependencies)
[dev-dependencies]
mockall = "0.12"
proptest = "1.0"
rstest = "0.18"
tokio-test = "0.4"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Migration Path from Current Stack

| Current | New | Migration Effort |
|---------|-----|------------------|
| `rusqlite` | `sqlx` | High (rewrite all queries) |
| `r2d2` | `sqlx::Pool` | Medium (connection handling) |
| Global `APP` | DI pattern | Medium (refactor instantiation) |
| `println!` | `tracing` | Low (drop-in replacement) |
| No migrations | `sqlx::migrate!` | Low (new feature) |
| No mocks | `mockall` | Medium (add test infrastructure) |

---

## Recommended Approach

### Phase 1: Database Layer (Week 1)
1. Add `sqlx` dependency
2. Create parallel `SqlxContentRepo` and `SqlxUserRepo`
3. Migrate queries one-by-one
4. Keep `rusqlite` temporarily for compatibility
5. Remove `rusqlite` once migration complete

### Phase 2: Testing (Week 1-2)
1. Add `mockall` for mocks
2. Add `rstest` for fixtures
3. Write unit tests for refactored code
4. Target 80%+ coverage

### Phase 3: CLI Tool (Week 2)
1. Create `iqrah-cli` binary target
2. Add `clap` for commands
3. Reuse core library
4. Ship debugging/testing tools

### Phase 4: Migrations (Week 2-3)
1. Set up `sqlx::migrate!`
2. Define migration files
3. Test rollback scenarios
4. Document migration process

---

## Resources

- **SQLx Book:** https://github.com/launchbadge/sqlx
- **Mockall Guide:** https://docs.rs/mockall/latest/mockall/
- **Clap Tutorial:** https://docs.rs/clap/latest/clap/_derive/
- **Tracing Guide:** https://tokio.rs/tokio/topics/tracing

---

## Decision: Approve This Stack?

**Recommendation:** ✅ **YES** - This is a production-ready, well-supported stack

**Risks:** Low - All libraries are mature, actively maintained, and widely used

**Effort:** ~40 story points to fully migrate, but worth the investment
