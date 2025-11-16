# Sprint 7 Implementation Gaps

**Status**: Sprint 7 architecture was partially implemented. The new crates exist, but critical integration and migration work remains incomplete.

**Discovery Date**: 2025-11-16
**Test Status**: 3 failing tests, 6 passing

---

## Executive Summary

Sprint 7's goal was to refactor Iqrah from a monolithic prototype to a production-ready, modular architecture with:
- 4 separate crates (iqrah-core, iqrah-storage, iqrah-api, iqrah-cli)
- Two-database architecture (content.db + user.db)
- 80%+ test coverage
- Zero SQL in business logic
- Migration framework

**Current Reality**: The new crates were scaffolded, but the Flutter app still uses the old monolithic code. Critical gaps prevent Sprint 8 from proceeding.

---

## Critical Gaps (Blocking Sprint 8)

### 1. Database Schema Mismatch ⚠️ CRITICAL

**Problem**: Two different content database schemas exist and conflict.

**Files in Conflict**:
- `rust/crates/iqrah-storage/src/content_schema.sql` (OLD - has `node_metadata` table)
- `rust/crates/iqrah-storage/migrations/01_content_schema.sql` (NEW - has `quran_text` and `translations` tables)

**Code Location**: `rust/crates/iqrah-storage/src/content/mod.rs:17`
```rust
// Currently uses OLD schema
let schema = include_str!("../content_schema.sql");
sqlx::raw_sql(schema).execute(&pool).await?;
```

**Repository Expects NEW Schema**: `rust/crates/iqrah-storage/src/content/repository.rs:57-80`
```rust
// Queries quran_text and translations tables (not node_metadata)
async fn get_quran_text(&self, node_id: &str) -> Result<Option<String>> {
    let row = query_as::<_, QuranTextRow>(
        "SELECT node_id, arabic FROM quran_text WHERE node_id = ?"
    )
    // ^^^ This table doesn't exist in the schema being used!
```

**Impact**:
- Integration tests fail: `test_content_repository_crud`, `test_two_database_integration`
- Repository methods will fail at runtime
- Cannot test content.db functionality

**Fix Required**:
- Remove `src/content_schema.sql`
- Update `init_content_db()` to use SQLx migrations like `init_user_db()` does
- Ensure migrations run in correct order

---

### 2. Migration Files Not in SQLx Format ⚠️ CRITICAL

**Problem**: Migration files exist but aren't in the format SQLx expects.

**Current Structure**:
```
migrations/
  01_content_schema.sql
  02_user_schema.sql
```

**SQLx Expects**:
```
migrations/
  20241116000001_content_schema.sql
  20241116000002_user_schema.sql
```

**Evidence**: `rust/crates/iqrah-storage/src/user/mod.rs:17`
```rust
// This works for user.db because it uses sqlx::migrate!
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

But the migration directory needs timestamp-prefixed files to work correctly.

**Impact**:
- Migrations may not run in the correct order
- SQLx migration tracking won't work properly
- CI/CD deployment will be unreliable

**Fix Required**:
- Rename migration files with timestamp prefixes
- Add migration metadata if needed
- Test migration ordering

---

### 3. User Database Migration Incomplete ⚠️ BLOCKING

**Problem**: Migration creates tables but doesn't initialize required data.

**Test Expectation**: `rust/crates/iqrah-storage/tests/integration_tests.rs:21-29`
```rust
// Test expects schema_version to be set
let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'schema_version'")
    .fetch_optional(&pool)
    .await
    .unwrap();
assert!(row.is_some(), "Migration v2 should have created app_settings table");
let version: String = row.unwrap().get("value");
assert_eq!(version, "2", "Schema version should be 2 after migrations");
```

**Current Migration**: `rust/crates/iqrah-storage/migrations/02_user_schema.sql:58-62`
```sql
-- Creates table but doesn't insert schema_version row
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
```

**Impact**:
- Test `test_user_db_initialization_and_migrations` fails
- No way to track schema version
- Future migrations cannot check version

**Fix Required**:
- Add a third migration: `20241116000003_initialize_settings.sql`
- Insert `schema_version = '2'` into `app_settings`

---

### 4. Flutter App Not Using New Crates ⚠️ CRITICAL

**Problem**: The new architecture exists but Flutter still uses the old monolithic code.

**Evidence**:

**Old Code Still Active**: `rust/src/lib.rs:4-10`
```rust
// All old modules still exported
pub mod cbor_import;
pub mod database;
pub mod exercises;
pub mod propagation;
pub mod repository;
pub mod sqlite_repo;
```

**API Still Uses Old Code**: `rust/src/api/mod.rs:32-34`
```rust
// Calls old app singleton, not new services
crate::app::init_app(db_path)?;
let service = &crate::app::app().service;
```

**New Crates Exist But Unused**:
- `rust/crates/iqrah-core/` ✅ Created
- `rust/crates/iqrah-storage/` ✅ Created
- `rust/crates/iqrah-api/` ✅ Created
- `rust/crates/iqrah-cli/` ✅ Created

But `rust/src/api/mod.rs` doesn't import ANY of these crates!

**Impact**:
- Sprint 7's architectural benefits not realized
- Cannot test business logic in isolation
- Technical debt not addressed
- Sprint 8's headless server cannot be built (needs clean API)

**Fix Required**:
- Update `iqrah-api` crate to be the actual Flutter bridge
- Migrate API functions from `rust/src/api/mod.rs` to `rust/crates/iqrah-api/src/lib.rs`
- Update root `Cargo.toml` to use `iqrah-api` as the cdylib
- Generate new Flutter bridge code
- Test Flutter integration
- Remove old code in `rust/src/`

---

## Major Gaps (High Priority)

### 5. Documentation Not Updated

**Problem**: Documentation still references old architecture.

**Outdated File**: `.github/copilot-instructions.md:19`
```markdown
5.  **Persistence (`rust/src/sqlite_repo.rs`)**: Data is stored in a SQLite database.
```

Should reference:
```markdown
5. **Persistence**: Separated into:
   - `rust/crates/iqrah-core/src/ports/`: Repository traits
   - `rust/crates/iqrah-storage/src/`: SQLx implementations
```

**Impact**:
- Future developers will be confused
- AI assistants will give wrong advice
- Onboarding time increased

**Fix Required**:
- Update `.github/copilot-instructions.md`
- Create `rust/crates/README.md` explaining new architecture
- Update main `README.md`

---

### 6. No CLI Documentation

**Problem**: `iqrah-cli` crate exists but no usage documentation.

**What Exists**: `rust/crates/iqrah-cli/src/main.rs` (assumed to exist)

**What's Missing**:
- No README in `rust/crates/iqrah-cli/`
- No examples of how to use it
- Not built in CI
- Not in PATH for developers

**Expected Functionality** (from Sprint 7 plan):
```bash
# Debug commands
iqrah debug get-node <NODE_ID>
iqrah debug get-state <USER_ID> <NODE_ID>
iqrah debug process-review <USER_ID> <NODE_ID> <GRADE>

# Exercise commands
iqrah exercise run <EXERCISE_TYPE> <NODE_ID>
```

**Impact**:
- Cannot debug database state easily
- Cannot test without Flutter UI
- Sprint 8's headless testing blocked

**Fix Required**:
- Create `rust/crates/iqrah-cli/README.md`
- Document all subcommands
- Add CLI build to CI
- Create installation instructions

---

### 7. Test Coverage Below Target

**Problem**: Sprint 7 goal was 80%+ test coverage, current status unknown.

**Current Test Status**:
```
Test result: FAILED. 6 passed; 3 failed
```

**Tests That Exist**:
- ✅ `rust/tests/import_test.rs`
- ✅ `rust/tests/propagation_tests.rs`
- ✅ `rust/crates/iqrah-core/src/services/learning_service_tests.rs`
- ✅ `rust/crates/iqrah-core/src/services/session_service_tests.rs`
- ❌ `rust/crates/iqrah-storage/tests/integration_tests.rs` (3 tests failing)

**Missing Tests** (per Sprint 7 plan):
- Property tests (PropTest)
- Unit tests for all core services
- Integration tests for propagation
- End-to-end tests

**Impact**:
- Unknown actual coverage
- Regressions possible
- Confidence low for refactoring

**Fix Required**:
- Fix failing tests first
- Run `cargo tarpaulin` or similar to measure coverage
- Add missing tests until 80%+ reached
- Add coverage reporting to CI

---

## Minor Gaps (Medium Priority)

### 8. No Content Database Migration Tool

**Problem**: No way to migrate existing user data from old schema to new schema.

**Sprint 7 Plan Expected**: A migration tool to convert:
- Old: Single `app.db` with `node_metadata` table
- New: Separate `content.db` + `user.db` with structured tables

**What Exists**: `rust/crates/iqrah-storage/src/migrations/mod.rs`

**What's Missing**:
- No migration runner tool
- No backup creation
- No rollback capability
- No data validation after migration

**Impact**:
- Existing users will lose data on update
- Cannot deploy to production safely
- Manual migration required

**Fix Required**:
- Create migration tool in `iqrah-cli`
- Test with real user data
- Add migration verification
- Document migration process

---

### 9. No CI Integration for New Crates

**Problem**: CI may not be testing the new crates.

**Current CI**: `.github/workflows/iqrah-knowledge-graph-ci.yml` (exists)

**Needs Verification**:
- Does CI run `cargo test --workspace`?
- Does CI build all 4 crates?
- Does CI check for compilation warnings?
- Does CI measure test coverage?

**Impact**:
- Broken code may be merged
- Tests may be skipped
- Coverage regression possible

**Fix Required**:
- Review and update CI workflow
- Ensure all crates tested
- Add coverage reporting
- Add compilation check

---

### 10. Content Database Migration Path Unclear

**Problem**: No clear strategy for how `content.db` gets created and updated.

**Questions Unanswered**:
1. Does `content.db` ship with the app as an asset?
2. Is it built from CBOR during first launch?
3. How are updates delivered (new schema versions)?
4. Who populates `quran_text` and `translations` tables?

**Current Code**: `rust/src/api/mod.rs:38-39`
```rust
// Imports from CBOR into... which database?
let import_stats = service.import_cbor_graph_from_bytes(kg_bytes).await?;
```

**Impact**:
- Cannot finalize schema
- Cannot build Sprint 8's headless server (needs real content.db)
- Update mechanism undefined

**Fix Required**:
- Document content.db creation pipeline
- Clarify CBOR import vs pre-built database
- Define update delivery mechanism

---

## Summary Statistics

### Implementation Progress

| Component | Status | Notes |
|-----------|--------|-------|
| ✅ iqrah-core crate | CREATED | Exists, some tests passing |
| ✅ iqrah-storage crate | CREATED | Exists, integration tests failing |
| ✅ iqrah-api crate | CREATED | Exists but not used |
| ✅ iqrah-cli crate | CREATED | Exists but undocumented |
| ❌ Flutter integration | NOT DONE | Still uses old code |
| ❌ Two-database setup | INCOMPLETE | Schema mismatch |
| ❌ Migration framework | BROKEN | Format issues |
| ❌ 80% test coverage | UNKNOWN | Likely below 50% |
| ❌ Documentation | OUTDATED | References old arch |
| ❌ Old code removal | NOT DONE | `rust/src/` still active |

**Estimated Completion**: ~35-40% of Sprint 7 goals achieved

---

## Recommended Fix Order

To unblock Sprint 8, fix gaps in this order:

### Phase 1: Fix Database Schema (1-2 days)
1. ✅ Rename migration files to SQLx format with timestamps
2. ✅ Remove `src/content_schema.sql`
3. ✅ Update `init_content_db()` to use SQLx migrations
4. ✅ Add third migration to initialize `app_settings`
5. ✅ Run all tests, verify they pass

### Phase 2: Integrate New Crates with Flutter (2-3 days)
1. ✅ Move API functions from `rust/src/api/mod.rs` to `iqrah-api` crate
2. ✅ Update root Cargo.toml to use `iqrah-api` as cdylib
3. ✅ Regenerate Flutter bridge code
4. ✅ Test Flutter app with new architecture
5. ✅ Remove old code from `rust/src/`

### Phase 3: Documentation & Testing (1 day)
1. ✅ Update `.github/copilot-instructions.md`
2. ✅ Create CLI README
3. ✅ Measure test coverage
4. ✅ Update main README

### Phase 4: Sprint 8 Preparation (0.5 days)
1. ✅ Verify iqrah-cli can query databases
2. ✅ Document current state
3. ✅ Create Sprint 8 starting point document

**Total Estimated Effort**: 4.5-6.5 days

---

## Sprint 8 Blockers

**Cannot start Sprint 8 until**:
1. ✅ All integration tests pass
2. ✅ Flutter app uses new crates
3. ✅ `content.db` and `user.db` work correctly
4. ✅ `iqrah-cli` is functional and documented

**Sprint 8 Requirement**: Build headless test server (`iqrah-server`)
- **Depends on**: Clean iqrah-core API (blocked until Flutter migration done)
- **Depends on**: Working two-database setup (blocked by schema issues)
- **Depends on**: CLI for scripting tests (blocked by lack of docs)

---

## Root Cause Analysis

**Why was Sprint 7 incomplete?**

1. **Scope Underestimation**: Refactoring + testing + migration was more work than planned
2. **Integration Complexity**: Separating crates is easy, integrating with Flutter is hard
3. **Testing Gaps**: Tests were written for new code but schemas didn't match
4. **Migration Complexity**: Database migrations are non-trivial, especially with SQLx
5. **Documentation Lag**: Code changes happened, docs didn't keep up

**Lessons for Future Sprints**:
- Always test integration with Flutter immediately, not at the end
- Database schema changes require migration testing with real data
- Update documentation in the same PR as code changes
- Smaller, incremental refactors are safer than big-bang rewrites

---

## Next Steps

1. **Immediate**: Review this gap analysis with the team
2. **Decision**: Should we finish Sprint 7 before starting Sprint 8? (Recommended: YES)
3. **Planning**: Break Phase 1-4 fixes into smaller tasks
4. **Execution**: Start with Phase 1 (database schema fixes)
5. **Validation**: After each phase, run full test suite and manual smoke tests

---

*Document Version: 1.0*
*Last Updated: 2025-11-16*
*Next Review: After Phase 1 completion*
