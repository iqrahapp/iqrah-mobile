# Sprint 7 Gap Remediation Plan

**Status**: Ready for Execution
**Created**: 2025-11-16
**Priority**: CRITICAL (Blocks Sprint 8)

---

## Quick Start

**Goal**: Fix critical gaps in Sprint 7 to unblock Sprint 8 (headless test server).

**Execution Order**:
1. Phase 1: Database Schema Fixes (START HERE)
2. Phase 2: Flutter Integration
3. Phase 3: Documentation & Testing
4. Phase 4: Sprint 8 Prep

**Estimated Time**: 4.5-6.5 days total

---

## Phase 1: Database Schema Fixes (1-2 days)

**Goal**: Make all integration tests pass by fixing schema mismatches and migration issues.

### Task 1.1: Rename Migrations to SQLx Format (30 min)

**Current**:
```
rust/crates/iqrah-storage/migrations/
  01_content_schema.sql
  02_user_schema.sql
```

**Required Format**:
```
rust/crates/iqrah-storage/migrations/
  20241116000001_content_schema.sql
  20241116000002_user_schema.sql
  20241116000003_initialize_settings.sql  # New
```

**Commands**:
```bash
cd rust/crates/iqrah-storage/migrations

# Rename existing migrations
mv 01_content_schema.sql 20241116000001_content_schema.sql
mv 02_user_schema.sql 20241116000002_user_schema.sql

# Verify
ls -la
```

**Verification**:
- Files have timestamp prefixes
- SQLx can detect them: `cargo sqlx migrate info`

---

### Task 1.2: Create Missing Settings Initialization Migration (15 min)

**Create**: `rust/crates/iqrah-storage/migrations/20241116000003_initialize_settings.sql`

**Content**:
```sql
-- Initialize app_settings with schema version
INSERT INTO app_settings (key, value) VALUES ('schema_version', '2');
```

**Verification**:
```bash
cd rust/crates/iqrah-storage
cargo test test_user_db_initialization_and_migrations -- --nocapture
```

Should output: ✅ Test passed

---

### Task 1.3: Remove Old Content Schema File (5 min)

**Problem**: `src/content_schema.sql` conflicts with new migrations.

**Commands**:
```bash
cd rust/crates/iqrah-storage
rm src/content_schema.sql

# Verify it's gone
ls src/*.sql
# Should show: No such file or directory (expected)
```

**Impact**: `init_content_db()` will break. Fix in next task.

---

### Task 1.4: Update init_content_db() to Use Migrations (45 min)

**File**: `rust/crates/iqrah-storage/src/content/mod.rs`

**Current Code** (lines 10-21):
```rust
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run schema creation
    let schema = include_str!("../content_schema.sql");
    sqlx::raw_sql(schema).execute(&pool).await?;

    Ok(pool)
}
```

**New Code**:
```rust
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations for content database
    // Note: Since content.db uses the same migrations folder, we need to ensure
    // only content-related migrations run. For now, we'll use the first migration.
    // TODO: Consider separating content and user migrations into different folders
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**Problem**: Both content and user databases share the same `migrations/` folder!

**Better Solution**: Separate migration folders

**Create**:
```
rust/crates/iqrah-storage/
  migrations_content/
    20241116000001_content_schema.sql
  migrations_user/
    20241116000001_user_schema.sql
    20241116000002_initialize_settings.sql
```

**Updated Code**:
```rust
// In content/mod.rs
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    Ok(pool)
}

// In user/mod.rs (update existing)
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!("./migrations_user")  // Changed from "./migrations"
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**Commands**:
```bash
cd rust/crates/iqrah-storage

# Create new directories
mkdir migrations_content migrations_user

# Move content migration
mv migrations/20241116000001_content_schema.sql migrations_content/

# Move user migrations
mv migrations/20241116000002_user_schema.sql migrations_user/20241116000001_user_schema.sql
mv migrations/20241116000003_initialize_settings.sql migrations_user/20241116000002_initialize_settings.sql

# Remove old migrations folder
rmdir migrations

# Update the code as shown above
```

**Verification**:
```bash
cargo test --workspace
```

All tests should pass! ✅

---

### Task 1.5: Fix Integration Test Assertions (30 min)

**File**: `rust/crates/iqrah-storage/tests/integration_tests.rs`

**Problem**: Test tries to insert into `node_metadata` which doesn't exist.

**Current Code** (lines 37-50):
```rust
// Insert test data manually
sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
    .execute(&pool)
    .await
    .unwrap();

sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'arabic', 'بِسْمِ')")
    .execute(&pool)
    .await
    .unwrap();

sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'translation', 'In the name')")
    .execute(&pool)
    .await
    .unwrap();
```

**Fixed Code**:
```rust
// Insert test data using NEW schema tables
sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
    .execute(&pool)
    .await
    .unwrap();

// Use quran_text table instead of node_metadata
sqlx::query("INSERT INTO quran_text VALUES ('node1', 'بِسْمِ')")
    .execute(&pool)
    .await
    .unwrap();

// Use translations table instead of node_metadata
sqlx::query("INSERT INTO translations (node_id, language_code, translation) VALUES ('node1', 'en', 'In the name')")
    .execute(&pool)
    .await
    .unwrap();
```

**Do the same** for lines 244-252 in `test_two_database_integration`

**Verification**:
```bash
cargo test test_content_repository_crud -- --nocapture
cargo test test_two_database_integration -- --nocapture
```

Both should pass! ✅

---

### Phase 1 Completion Checklist

- [ ] All migrations renamed with timestamps
- [ ] Migrations separated into `migrations_content/` and `migrations_user/`
- [ ] `init_content_db()` uses SQLx migrations
- [ ] `init_user_db()` uses correct migration path
- [ ] `app_settings` table initialized with schema_version
- [ ] Integration tests updated to use new schema
- [ ] All 9 tests pass: `cargo test --workspace`

**When complete, commit**:
```bash
git add .
git commit -m "fix(storage): resolve database schema conflicts and migration issues

- Separate content and user migrations into dedicated folders
- Update init functions to use SQLx migration framework
- Fix integration tests to use new schema (quran_text, translations)
- Initialize app_settings with schema_version
- All tests now passing (9/9)

Closes critical gaps from SPRINT_7_GAPS.md #1, #2, #3"
```

---

## Phase 2: Flutter Integration (2-3 days)

**Goal**: Make Flutter app use the new crates instead of old monolithic code.

### Task 2.1: Audit Current API Surface (1 hour)

**Create**: `MIGRATION_CHECKLIST.md`

**List all functions** in `rust/src/api/mod.rs` that Flutter calls:
```
- setup_database
- setup_database_in_memory
- get_exercises
- process_review
- get_debug_stats
- reseed_database
- refresh_priority_scores
- get_session_preview
- search_nodes
- fetch_node_with_metadata
- get_existing_session
- get_dashboard_stats
- clear_session
- get_exercises_for_node
- get_available_surahs
```

For each function, document:
1. Current implementation (old code path)
2. Required new code path (which service/repository)
3. Dependencies (what needs to exist first)

---

### Task 2.2: Implement Services in iqrah-api Crate (1-2 days)

**Strategy**: Incremental migration, one function at a time.

**Step 1**: Create service initialization in `iqrah-api`

**File**: `rust/crates/iqrah-api/src/lib.rs`

**Add**:
```rust
use iqrah_core::{LearningService, SessionService};
use iqrah_storage::{init_content_db, init_user_db, SqliteContentRepository, SqliteUserRepository};
use once_cell::sync::OnceCell;
use std::sync::Arc;

static SERVICES: OnceCell<AppServices> = OnceCell::new();

pub struct AppServices {
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
}

pub async fn init_services(content_db_path: &str, user_db_path: &str) -> anyhow::Result<()> {
    let content_pool = init_content_db(content_db_path).await?;
    let user_pool = init_user_db(user_db_path).await?;

    let content_repo = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo = Arc::new(SqliteUserRepository::new(user_pool));

    let learning_service = Arc::new(LearningService::new(
        content_repo.clone(),
        user_repo.clone(),
    ));

    let session_service = Arc::new(SessionService::new(
        content_repo.clone(),
        user_repo.clone(),
    ));

    SERVICES.set(AppServices {
        learning_service,
        session_service,
    }).map_err(|_| anyhow::anyhow!("Services already initialized"))?;

    Ok(())
}

pub fn services() -> &'static AppServices {
    SERVICES.get().expect("Services not initialized. Call init_services first")
}
```

**Step 2**: Migrate one function at a time

**Example**: Migrate `get_debug_stats`

**Before** (`rust/src/api/mod.rs`):
```rust
pub async fn get_debug_stats(user_id: String) -> Result<DebugStats> {
    crate::app::app().service.get_debug_stats(&user_id).await
}
```

**After** (`rust/crates/iqrah-api/src/lib.rs`):
```rust
pub async fn get_debug_stats(user_id: String) -> Result<DebugStats> {
    services().learning_service.get_debug_stats(&user_id).await
}
```

**Repeat** for all 15 functions.

---

### Task 2.3: Update Root Cargo.toml (30 min)

**File**: `rust/Cargo.toml`

**Current**:
```toml
[package]
name = "iqrah"  # Root package
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]
```

**Updated**:
```toml
[package]
name = "iqrah"
version = "0.1.0"
edition = "2021"

# THIS IS NOW JUST A WORKSPACE ROOT
# The actual library is in crates/iqrah-api

[workspace]
members = [
    "crates/iqrah-core",
    "crates/iqrah-storage",
    "crates/iqrah-api",      # This becomes the cdylib
    "crates/iqrah-cli",
]
resolver = "2"

# ... keep workspace.dependencies ...
```

**File**: `rust/crates/iqrah-api/Cargo.toml`

**Add**:
```toml
[package]
name = "iqrah-api"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]  # Moved from root

[dependencies]
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }
flutter_rust_bridge = "2.0"
anyhow = { workspace = true }
tokio = { workspace = true }
once_cell = { workspace = true }
```

---

### Task 2.4: Regenerate Flutter Bridge (15 min)

**Commands**:
```bash
# Update flutter_rust_bridge config to point to new location
# Edit flutter_rust_bridge.yaml

# Then regenerate
flutter_rust_bridge_codegen generate
```

**File**: `flutter_rust_bridge.yaml`

**Update**:
```yaml
# Change rust_input from rust/src/api to:
rust_input: rust/crates/iqrah-api/src/lib.rs

# Keep dart_output the same
dart_output: lib/rust_bridge/
```

---

### Task 2.5: Test Flutter App (1 hour)

**Commands**:
```bash
# Build Rust library
cd rust
cargo build --release --package iqrah-api

# Run Flutter app
flutter run
```

**Manual Tests**:
1. Launch app
2. Import data (should work with new databases)
3. Generate exercises
4. Complete a review
5. Check stats

**If issues**, check logs and fix service calls.

---

### Task 2.6: Remove Old Code (1 hour)

**Only after Flutter app works!**

**Commands**:
```bash
cd rust/src

# Move old files to archive
mkdir ../archive
mv api ../archive/
mv app.rs ../archive/
mv cbor_import.rs ../archive/
mv database.rs ../archive/
mv exercises.rs ../archive/
mv propagation.rs ../archive/
mv repository.rs ../archive/
mv sqlite_repo.rs ../archive/

# Keep only:
# - lib.rs (update to just export iqrah-api)
# - frb_generated.rs (auto-generated)
```

**Update**: `rust/src/lib.rs`
```rust
// Re-export iqrah-api as the public interface
pub use iqrah_api::*;

mod frb_generated;
```

**Test again**:
```bash
flutter run
```

Should still work! ✅

---

### Phase 2 Completion Checklist

- [ ] API audit complete, all functions documented
- [ ] `iqrah-api` crate implements all API functions
- [ ] Root Cargo.toml updated to use iqrah-api as cdylib
- [ ] Flutter bridge regenerated
- [ ] Flutter app tested and working with new crates
- [ ] Old code archived
- [ ] Final smoke test passes

**When complete, commit**:
```bash
git add .
git commit -m "refactor: migrate Flutter app to use new modular architecture

- Implement all API functions in iqrah-api crate
- Update build configuration to use iqrah-api as cdylib
- Regenerate Flutter bridge for new code paths
- Archive old monolithic code
- All features verified working

Closes critical gap from SPRINT_7_GAPS.md #4"
```

---

## Phase 3: Documentation & Testing (1 day)

### Task 3.1: Update Copilot Instructions (30 min)

**File**: `.github/copilot-instructions.md`

**Replace** Architecture section (lines 1-30) with:

```markdown
## Architecture Overview (Post-Sprint 7)

This project uses a **modular Rust core** with **hexagonal architecture**:

### Crates
- **`iqrah-core/`**: Pure business logic (FSRS, propagation, services)
- **`iqrah-storage/`**: Database implementations (SQLx, SQLite)
- **`iqrah-api/`**: Flutter bridge (FRB, API layer)
- **`iqrah-cli/`**: Developer CLI tool

### Data Flow
1. **UI Layer** (`lib/pages/`) → Flutter widgets
2. **State Management** (`lib/providers/`) → Riverpod providers
3. **Bridge Layer** (`iqrah-api`) → Rust API via FRB
4. **Service Layer** (`iqrah-core`) → LearningService, SessionService
5. **Repository Layer** (`iqrah-storage`) → SQLite via SQLx

### Databases
- **`content.db`**: Read-only Qur'anic knowledge graph
- **`user.db`**: Read-write user progress and stats

### Key Principles
- **Dependency Inversion**: Core depends on traits, not implementations
- **Testability**: Mock repositories for unit tests
- **Separation of Concerns**: Content ≠ User Data
```

---

### Task 3.2: Create CLI README (45 min)

**File**: `rust/crates/iqrah-cli/README.md`

**Content**:
```markdown
# Iqrah CLI

Developer tool for debugging and testing the Iqrah learning system.

## Installation

```bash
cd rust
cargo install --path crates/iqrah-cli
```

## Usage

### Debug Commands

**Get node details**:
```bash
iqrah debug get-node VERSE:2:255
```

**Get user memory state**:
```bash
iqrah debug get-state default_user WORD:2:255:1
```

**Simulate a review**:
```bash
iqrah debug process-review default_user WORD:2:255:1 Good
```

### Database Commands

**Initialize databases**:
```bash
iqrah db init --content content.db --user user.db
```

**Run migrations**:
```bash
iqrah db migrate --database user.db
```

**Import knowledge graph**:
```bash
iqrah db import --cbor iqrah-graph-v1.0.1.cbor.zst --output content.db
```

### Testing Commands

**Generate a test session**:
```bash
iqrah test session --user default_user --limit 10
```

**Verify propagation**:
```bash
iqrah test propagation --node VERSE:2:255
```

## Configuration

Create `.iqrah-cli.toml` in your home directory:

```toml
[databases]
content_db = "/path/to/content.db"
user_db = "/path/to/user.db"

[defaults]
user_id = "default_user"
```
```

**Implement** these commands in the CLI crate if they don't exist yet.

---

### Task 3.3: Measure Test Coverage (1 hour)

**Install** coverage tool:
```bash
cargo install cargo-tarpaulin
```

**Run** coverage:
```bash
cd rust
cargo tarpaulin --workspace --out Html --output-dir coverage
```

**Review** `coverage/index.html`

**Target Coverage** (Sprint 7 goal):
- iqrah-core: 90%+
- iqrah-storage: 80%+
- iqrah-api: 60%+

**If below target**, add tests until goals met.

---

### Task 3.4: Update Main README (30 min)

**File**: `README.md`

**Add** section after "Getting Started":

```markdown
## Architecture

Iqrah uses a modular Rust backend with clean architecture principles:

- **Core Business Logic** (`rust/crates/iqrah-core/`): Platform-agnostic learning algorithms
- **Data Layer** (`rust/crates/iqrah-storage/`): SQLite repositories
- **API Layer** (`rust/crates/iqrah-api/`): Flutter bridge
- **CLI Tool** (`rust/crates/iqrah-cli/`): Developer utilities

See [Architecture Docs](./.github/sprint7/03-ARCHITECTURE-BLUEPRINT.md) for details.

## Development

### Running Tests

```bash
cd rust
cargo test --workspace
```

### Using the CLI

```bash
cd rust
cargo run --package iqrah-cli -- debug get-node VERSE:1:1
```

### Measuring Coverage

```bash
cargo tarpaulin --workspace
```
```

---

### Phase 3 Completion Checklist

- [ ] Copilot instructions updated
- [ ] CLI README created
- [ ] Test coverage measured (report generated)
- [ ] Main README updated
- [ ] All docs reviewed for accuracy

**When complete, commit**:
```bash
git add .
git commit -m "docs: update documentation for Sprint 7 architecture

- Update copilot instructions to reflect new crate structure
- Create comprehensive CLI README
- Add architecture section to main README
- Include test coverage reporting

Closes gap from SPRINT_7_GAPS.md #5, #6"
```

---

## Phase 4: Sprint 8 Preparation (0.5 days)

### Task 4.1: Verify CLI Functionality (1 hour)

**Test each command**:
```bash
# Initialize test databases
iqrah db init --content /tmp/test_content.db --user /tmp/test_user.db

# Import sample data (if CBOR available)
iqrah db import --cbor assets/iqrah-graph-v1.0.1.cbor.zst --output /tmp/test_content.db

# Query a node
iqrah debug get-node VERSE:1:1

# Check user state
iqrah debug get-state test_user VERSE:1:1

# Process a review
iqrah debug process-review test_user VERSE:1:1 Good
```

**Document** any missing commands needed for Sprint 8.

---

### Task 4.2: Create Sprint 8 Starting Point Document (1 hour)

**File**: `SPRINT_8_READY.md`

**Content**:
```markdown
# Sprint 8 Readiness Status

**Date**: 2025-11-16
**Sprint 7 Status**: COMPLETE ✅

## Prerequisites Met

- ✅ All integration tests pass (9/9)
- ✅ Flutter app uses new modular architecture
- ✅ Two-database setup working (content.db + user.db)
- ✅ iqrah-cli functional and documented
- ✅ Test coverage: [XX]% (target: 80%)

## Available Infrastructure for Sprint 8

### Crates Ready to Use
- `iqrah-core`: Clean service APIs for learning and sessions
- `iqrah-storage`: Repository implementations with migrations
- `iqrah-api`: Flutter bridge (existing API surface)
- `iqrah-cli`: CLI tool with debug commands

### Database Setup
- Content DB: Migrations in `migrations_content/`
- User DB: Migrations in `migrations_user/`
- Both support in-memory mode for testing

### CLI Commands Available
```bash
iqrah debug get-node <NODE_ID>
iqrah debug get-state <USER_ID> <NODE_ID>
iqrah debug process-review <USER_ID> <NODE_ID> <GRADE>
```

## Sprint 8 Implementation Plan

### New Crate: iqrah-server

**Location**: `rust/crates/iqrah-server/`

**Dependencies**:
```toml
iqrah-core = { path = "../iqrah-core" }
iqrah-storage = { path = "../iqrah-storage" }
axum = "0.7"
tokio-tungstenite = "0.21"
serde_json = "1.0"
```

**Entry Point**: `src/main.rs`

**Endpoints to Implement**:
- `GET /health`: Health check
- `GET /debug/node/:node_id`: Get node details
- `GET /debug/user/:user_id/state/:node_id`: Get memory state
- `POST /debug/user/:user_id/review`: Process review
- `WS /session`: WebSocket for interactive sessions

### Test Scripts

**Location**: `rust/tests/headless/`

**Scripts to Create**:
- `test_mcq_flow.sh`: Test stateless MCQ review
- `test_memorization_mvp.sh`: Test WebSocket memorization session

### CI Integration

**File**: `.github/workflows/headless_tests.yml`

**Jobs**:
1. Build iqrah-server
2. Start server in background
3. Run test scripts
4. Report results

## Next Actions

1. Create `iqrah-server` crate scaffold
2. Implement REST endpoints
3. Implement WebSocket protocol
4. Extend `iqrah-cli` with WebSocket client
5. Write test scripts
6. Add CI workflow

## Estimated Timeline

- Server implementation: 2-3 days
- CLI extension: 1 day
- Test scripts: 1 day
- CI integration: 0.5 days

**Total**: 4.5-5.5 days
```

---

### Task 4.3: Final Verification (30 min)

**Run full test suite**:
```bash
cd rust
cargo test --workspace --verbose
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

**All should pass** without errors.

**Run Flutter integration test**:
```bash
flutter test integration_test
```

**Should pass** with no regressions.

---

### Phase 4 Completion Checklist

- [ ] CLI verified functional
- [ ] Sprint 8 readiness document created
- [ ] Full test suite passing
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Integration tests pass

**When complete, commit**:
```bash
git add .
git commit -m "chore: verify Sprint 7 completion and prepare for Sprint 8

- Verify CLI functionality for headless testing
- Document Sprint 8 starting point and prerequisites
- Run full test suite and linting checks
- All prerequisites met for Sprint 8 implementation

Sprint 7 gaps resolved. Ready for Sprint 8."
```

---

## Success Criteria

Sprint 7 gaps are RESOLVED when:

✅ **Database**:
- All 9 integration tests pass
- Migrations run correctly for both databases
- Schema matches repository implementations

✅ **Architecture**:
- Flutter app uses new crates (iqrah-core, iqrah-storage, iqrah-api)
- Old monolithic code removed/archived
- Clean separation of concerns

✅ **Testing**:
- Test coverage ≥ 80% overall
- All tests pass in CI
- No clippy warnings

✅ **Documentation**:
- Architecture docs updated
- CLI fully documented
- README reflects new structure

✅ **Sprint 8 Ready**:
- CLI can query databases
- Services are testable
- Foundation ready for headless server

---

## Rollback Plan

If major issues arise during Phase 2 (Flutter integration):

1. **Revert Flutter bridge**: Use old API temporarily
2. **Keep new crates**: They're self-contained
3. **Fix issues**: Iterate on new code
4. **Re-attempt**: Try integration again

**Key**: New crates don't break existing code. Integration is additive.

---

## Communication

**Daily Updates**: Post progress in project channel
**Blockers**: Flag immediately if stuck > 2 hours
**Questions**: Ask in team chat or create GitHub issue

---

## Post-Completion

After all 4 phases complete:

1. **Code Review**: Request review from team
2. **Merge**: Create PR to main branch
3. **Deploy**: Test with staging environment
4. **Sprint 8**: Begin headless server implementation

---

*Plan Version: 1.0*
*Last Updated: 2025-11-16*
*Owner: AI Agent (with human review)*
