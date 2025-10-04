# Sprint 7: Migration Strategy

**Date:** 2025-10-04
**Purpose:** Step-by-step plan to migrate from current architecture to production-ready system

---

## Migration Philosophy

**Zero Downtime, Zero Data Loss**
- Keep app functional during migration
- Preserve all user data
- Rollback capability at each step
- Test extensively before deletion

---

## Pre-Migration Checklist

### 1. Backup Current State
```bash
# Backup existing database
cp ~/Documents/iqrah.db ~/Documents/iqrah_backup_$(date +%Y%m%d).db

# Backup codebase
git tag pre-sprint7-migration
git push origin pre-sprint7-migration
```

### 2. Data Audit
```sql
-- Count current data
SELECT 'nodes' as table_name, COUNT(*) as count FROM nodes
UNION ALL
SELECT 'user_memory_states', COUNT(*) FROM user_memory_states
UNION ALL
SELECT 'propagation_events', COUNT(*) FROM propagation_events;
```

Expected output (for validation later):
- nodes: ~50,000
- user_memory_states: varies (user-dependent)
- propagation_events: varies

---

## Phase 1: Create Workspace Structure

### Step 1.1: Initialize Cargo Workspace
```bash
cd /home/shared/ws/iqrah/rust

# Create workspace Cargo.toml
cat > Cargo.toml <<'EOF'
[workspace]
members = [
    "crates/iqrah-core",
    "crates/iqrah-storage",
    "crates/iqrah-api",
    "crates/iqrah-cli",
]
resolver = "2"

[workspace.dependencies]
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "sqlite", "macros"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
EOF
```

### Step 1.2: Create Crate Skeletons
```bash
# Create crate directories
mkdir -p crates/{iqrah-core,iqrah-storage,iqrah-api,iqrah-cli}/src

# iqrah-core
cat > crates/iqrah-core/Cargo.toml <<'EOF'
[package]
name = "iqrah-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
serde = { workspace = true }
chrono = { workspace = true }
fsrs = "1.3"

[dev-dependencies]
mockall = "0.12"
proptest = "1.0"
tokio-test = "0.4"
EOF

# iqrah-storage
cat > crates/iqrah-storage/Cargo.toml <<'EOF'
[package]
name = "iqrah-storage"
version = "0.1.0"
edition = "2021"

[dependencies]
iqrah-core = { path = "../iqrah-core" }
sqlx = { workspace = true }
anyhow = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
EOF

# iqrah-api
cat > crates/iqrah-api/Cargo.toml <<'EOF'
[package]
name = "iqrah-api"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }
flutter_rust_bridge = "2.0"
anyhow = { workspace = true }
tokio = { workspace = true }
once_cell = "1.19"
EOF

# iqrah-cli
cat > crates/iqrah-cli/Cargo.toml <<'EOF'
[package]
name = "iqrah-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "iqrah"
path = "src/main.rs"

[dependencies]
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }
clap = { version = "4", features = ["derive"] }
tokio = { workspace = true }
anyhow = { workspace = true }
EOF
```

### Step 1.3: Test Workspace Build
```bash
cargo check --workspace
```

---

## Phase 2: Extract Domain Models (Week 1, Days 1-2)

### Step 2.1: Move Core Types to iqrah-core
```bash
# Copy existing domain types
cp src/exercises.rs crates/iqrah-core/src/domain/exercises.rs
cp src/fsrs_utils.rs crates/iqrah-core/src/domain/scheduler.rs
```

Edit files to remove database dependencies:
- Remove all `rusqlite` imports
- Remove SQL queries
- Keep only pure logic

### Step 2.2: Define Repository Traits
Create `crates/iqrah-core/src/ports/content_repository.rs`:
```rust
use async_trait::async_trait;
use crate::domain::*;

#[async_trait]
pub trait ContentRepository: Send + Sync {
    async fn get_node(&self, id: &NodeId) -> Result<Node>;
    async fn get_quran_text(&self, id: &NodeId) -> Result<String>;
    async fn get_translation(&self, id: &NodeId, lang: &str) -> Result<String>;
    // ... (from architecture blueprint)
}
```

Create `crates/iqrah-core/src/ports/user_repository.rs`:
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_memory_state(&self, user_id: &str, node_id: &NodeId) -> Result<Option<MemoryState>>;
    // ... (from architecture blueprint)
}
```

### Step 2.3: Implement Services
Create `crates/iqrah-core/src/services/learning_service.rs`:
```rust
pub struct LearningService {
    content_repo: Arc<dyn ContentRepository>,
    user_repo: Arc<dyn UserRepository>,
    scheduler: Arc<dyn Scheduler>,
}

// Implement process_review, get_due_items, etc.
// (from architecture blueprint)
```

**Validation:**
```bash
cd crates/iqrah-core
cargo test
```

---

## Phase 3: Implement Storage Layer (Week 1, Days 3-5)

### Step 3.1: Create Database Schemas

Create `crates/iqrah-storage/migrations/00001_content_schema.sql`:
```sql
-- Content DB schema (from 02-DATABASE-SCHEMA-DESIGN.md)
CREATE TABLE nodes (...);
CREATE TABLE edges (...);
CREATE TABLE quran_text (...);
-- ... etc
```

Create `crates/iqrah-storage/migrations/00002_user_schema.sql`:
```sql
-- User DB schema
CREATE TABLE user_memory_states (...);
CREATE TABLE review_history (...);
-- ... etc
```

### Step 3.2: Implement SQLx Repositories

Create `crates/iqrah-storage/src/content/repository.rs`:
```rust
use sqlx::SqlitePool;
use iqrah_core::ports::ContentRepository;

pub struct SqliteContentRepository {
    pool: SqlitePool,
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    // Implement all trait methods with sqlx::query! macros
    // (from architecture blueprint)
}
```

Create `crates/iqrah-storage/src/user/repository.rs`:
```rust
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    // Implement all trait methods
}
```

**Validation:**
```bash
cd crates/iqrah-storage
cargo test --test integration_tests
```

### Step 3.3: Data Migration Tool

Create `crates/iqrah-storage/src/migration.rs`:
```rust
pub async fn migrate_from_old_db(
    old_db_path: &str,
    new_content_db: &SqlitePool,
    new_user_db: &SqlitePool,
) -> Result<()> {
    let old_pool = SqlitePool::connect(old_db_path).await?;

    // 1. Migrate content data
    let nodes: Vec<OldNode> = sqlx::query_as("SELECT * FROM nodes")
        .fetch_all(&old_pool)
        .await?;

    for node in nodes {
        sqlx::query!(
            "INSERT INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)",
            node.id,
            node.node_type,
            node.created_at
        )
        .execute(new_content_db)
        .await?;

        // Migrate metadata to new structured tables
        migrate_node_metadata(&old_pool, new_content_db, &node.id).await?;
    }

    // 2. Migrate user data
    let memory_states: Vec<OldMemoryState> = sqlx::query_as(
        "SELECT * FROM user_memory_states"
    )
    .fetch_all(&old_pool)
    .await?;

    for state in memory_states {
        sqlx::query!(
            "INSERT INTO user_memory_states (...) VALUES (...)",
            state.user_id,
            state.node_id,
            // ... all fields
        )
        .execute(new_user_db)
        .await?;
    }

    // 3. Migrate propagation log
    // ... similar pattern

    println!("Migration complete!");
    println!("  Nodes: {}", nodes.len());
    println!("  Memory states: {}", memory_states.len());

    Ok(())
}
```

---

## Phase 4: Update API Layer (Week 2, Days 1-2)

### Step 4.1: Refactor iqrah-api

Update `crates/iqrah-api/src/lib.rs`:
```rust
use iqrah_core::services::LearningService;
use iqrah_storage::{SqliteContentRepository, SqliteUserRepository};

pub struct AppState {
    learning_service: Arc<LearningService>,
}

pub async fn init_app(
    content_db_path: String,
    user_db_path: String,
) -> Result<String> {
    // Connect to databases
    let content_pool = SqlitePool::connect(&content_db_path).await?;
    let user_pool = SqlitePool::connect(&user_db_path).await?;

    // Run migrations on user.db
    sqlx::migrate!("../iqrah-storage/migrations/user")
        .run(&user_pool)
        .await?;

    // Create repositories
    let content_repo = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo = Arc::new(SqliteUserRepository::new(user_pool));
    let scheduler = Arc::new(FsrsScheduler::default());

    // Create service
    let learning_service = Arc::new(LearningService::new(
        content_repo,
        user_repo,
        scheduler,
    ));

    // Store in global state
    APP.set(AppState { learning_service })?;

    Ok("App initialized".to_string())
}

// Keep all existing FRB functions, but delegate to services
pub async fn get_exercises(...) -> Result<Vec<Exercise>> {
    let app = APP.get().ok_or_else(|| anyhow!("Not initialized"))?;
    app.learning_service.get_due_items(...).await
}
```

### Step 4.2: Update Flutter to use new API

Update `lib/main.dart`:
```dart
Future<void> main() async {
    // ...
    final contentDbPath = await getContentDatabasePath();
    final userDbPath = await getUserDatabasePath();

    // NEW: Initialize with two databases
    final initMsg = await initApp(
        contentDbPath: contentDbPath,
        userDbPath: userDbPath,
    );

    print(initMsg);
    runApp(const ProviderScope(child: MyApp()));
}
```

Create helper functions:
```dart
Future<String> getContentDatabasePath() async {
    final docsDir = await getApplicationDocumentsDirectory();
    return '${docsDir.path}/content.db';
}

Future<String> getUserDatabasePath() async {
    final docsDir = await getApplicationDocumentsDirectory();
    return '${docsDir.path}/user.db';
}
```

---

## Phase 5: One-Time Data Migration (Week 2, Day 3)

### Step 5.1: Create Migration Command in CLI

Add to `crates/iqrah-cli/src/commands/migrate.rs`:
```rust
pub async fn migrate_old_database(
    old_db_path: String,
    content_db_path: String,
    user_db_path: String,
) -> Result<()> {
    println!("Starting migration from {}", old_db_path);

    let content_pool = SqlitePool::connect(&content_db_path).await?;
    let user_pool = SqlitePool::connect(&user_db_path).await?;

    // Run migrations
    iqrah_storage::migrate_from_old_db(&old_db_path, &content_pool, &user_pool).await?;

    // Validate
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&content_pool)
        .await?;

    let state_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_memory_states")
        .fetch_one(&user_pool)
        .await?;

    println!("✅ Migration successful!");
    println!("   Nodes: {}", node_count);
    println!("   Memory states: {}", state_count);

    Ok(())
}
```

### Step 5.2: Execute Migration
```bash
# Build CLI
cargo build --bin iqrah --release

# Run migration
./target/release/iqrah migrate \
    --old-db ~/Documents/iqrah.db \
    --content-db ~/Documents/content.db \
    --user-db ~/Documents/user.db

# Backup old database (don't delete yet!)
mv ~/Documents/iqrah.db ~/Documents/iqrah_old.db.bak
```

### Step 5.3: Validation
```bash
# Check content.db
sqlite3 ~/Documents/content.db "SELECT COUNT(*) FROM nodes"
# Should match old count

# Check user.db
sqlite3 ~/Documents/user.db "SELECT COUNT(*) FROM user_memory_states"
# Should match old count

# Check new schema
sqlite3 ~/Documents/user.db "PRAGMA user_version"
# Should be 1 (first migration)
```

---

## Phase 6: Testing & Validation (Week 2, Days 4-5)

### Integration Tests
```bash
cd /home/shared/ws/iqrah/rust
cargo test --workspace --all-features
```

### End-to-End Test
```bash
# Run the app with new databases
flutter run

# Test key workflows:
# 1. Start session -> should work
# 2. Complete reviews -> should update stats
# 3. Check propagation -> should log correctly
# 4. Close and reopen -> should resume session
```

### Performance Benchmarks
```bash
./target/release/iqrah bench session --iterations 100
# Target: < 50ms per session generation
```

---

## Phase 7: Cleanup (Week 3)

### Step 7.1: Remove Old Code
```bash
# Once 100% validated, delete old files
rm rust/src/sqlite_repo.rs
rm rust/src/repository.rs  # old version
rm rust/src/database.rs    # old schema

# Move to new workspace structure entirely
```

### Step 7.2: Update Documentation
- README.md (new database structure)
- ARCHITECTURE.md (new design)
- API documentation

---

## Rollback Plan

If issues arise during migration:

### Rollback Step 1: Keep Old Code
```bash
# Old code stays in git history
git checkout pre-sprint7-migration
```

### Rollback Step 2: Restore Old Database
```bash
cp ~/Documents/iqrah_old.db.bak ~/Documents/iqrah.db
```

### Rollback Step 3: Revert Flutter
```dart
// Revert to single database init
final initMsg = await setupDatabase(dbPath: dbPath, kgBytes: bytes);
```

---

## Success Criteria

✅ **Data Integrity**
- All nodes migrated: `SELECT COUNT(*) FROM nodes` matches
- All user states migrated: `SELECT COUNT(*) FROM user_memory_states` matches
- Propagation log intact

✅ **Functionality**
- All existing features work identically
- No regressions
- Stats still update

✅ **Performance**
- Session generation faster (target: 2x improvement)
- user.db smaller (target: 90% reduction for new users)

✅ **Code Quality**
- Tests pass: `cargo test --workspace`
- No compilation warnings
- FRB bindings regenerated

---

## Timeline Summary

| Phase | Duration | Description |
|-------|----------|-------------|
| 1. Workspace setup | 0.5 days | Create crate structure |
| 2. Domain extraction | 1.5 days | Move models, define traits |
| 3. Storage layer | 3 days | SQLx repos, migrations |
| 4. API update | 1.5 days | Wire up new services |
| 5. Data migration | 1 day | One-time migration |
| 6. Testing | 1.5 days | Integration & E2E tests |
| 7. Cleanup | 1 day | Remove old code, docs |

**Total: ~10-12 days (2-2.5 weeks)**

---

Next: See `05-TESTING-STRATEGY.md` for comprehensive testing approach
