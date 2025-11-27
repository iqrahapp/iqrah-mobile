# Task 1.7: Consolidate Draft Migrations into Final Schema

## Metadata

- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 4-5 hours
- **Dependencies:** None (Can be done in parallel with Task 1.6 - different databases)
- **Agent Type:** Database Schema Consolidation + Testing
- **Parallelizable:** Yes (with Task 1.6 - affects user.db, not content.db)

## Goal

Consolidate the 9 iterative migration files in `migrations_user/` into a single, clean final migration file that represents the production schema. Ensure all tests continue to pass with the consolidated migration while maintaining data integrity and full feature support.

## Context

During development (Nov 16-26), the user database schema evolved through multiple iterations:

1. **Nov 16**: Initial schema (node_id as TEXT)
2. **Nov 17**: Added content_keys table, scheduler v2 bandit state, test data
3. **Nov 24-25**: Added schema_version tracking (duplicated)
4. **Nov 26**: Converted node_id from TEXT to INTEGER (i64 refactoring)

**Current State**: 9 migration files representing this evolution
**Problem**: Multiple draft migrations create unnecessary complexity and historical clutter
**Opportunity**: Since no production data exists, we can consolidate into one clean migration

**Why This Matters:**
- Production databases should start with a clean, single schema migration (not historical iterations)
- Multiple migrations increase maintenance burden and risk of migration bugs
- Consolidated schema makes it easier to understand the final state
- Matches the approach taken for content.db (single unified migration)
- Reduces complexity for new developers understanding the database structure

**Related Documentation:**
- [Two-Database Architecture](/CLAUDE.md) - User DB design principles
- [Task 1.6](/docs/todo/production-ready-tasks/task-1.6-schema-v2.1-and-test-data-separation.md) - Schema v2.1 redesign + test data separation (content DB)

## Current State

**Location:** `rust/crates/iqrah-storage/migrations_user/`

**Current Files (9 total):**

```
20241116000001_user_schema.sql                    (62 lines) - Initial schema
20241116000002_initialize_settings.sql            (4 lines)  - Schema version init
20241117000001_content_keys.sql                   (94 lines) - Content keys + scheduler v2 tables + test data
20241117000002_scheduler_v2_bandit.sql            (44 lines) - Bandit state table
20241117000003_scheduler_v2_sample_user_data.sql  (53 lines) - Sample test data
20241117000004_larger_test_user_data.sql          (31 lines) - Extended test data
20241124000001_add_schema_version.sql             (10 lines) - Schema version table (first attempt)
20241125000001_add_schema_version.sql             (10 lines) - Schema version table (duplicate)
20241126000002_convert_content_key_to_integer.sql (72 lines) - TEXT → INTEGER conversion
```

**Final Schema State (after all migrations):**

```sql
-- Core FSRS + Energy tracking
user_memory_states (
  user_id TEXT,
  content_key INTEGER,  ← FINAL: i64 format
  stability, difficulty, energy, last_reviewed, due_at, review_count
)

-- Session state
session_state (
  content_key INTEGER,  ← FINAL: i64 format
  session_order
)

-- Energy propagation tracking
propagation_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_content_key INTEGER,  ← FINAL: i64 format
  event_timestamp
)

propagation_details (
  id, event_id, target_content_key INTEGER, energy_change, path, reason
)

-- App state
user_stats (key TEXT, value TEXT)
app_settings (key TEXT, value TEXT)

-- Scheduler v2 Bandit state (Thompson Sampling)
user_bandit_state (
  user_id TEXT,
  goal_group TEXT,
  profile_name TEXT,
  successes REAL, failures REAL, last_updated
)

-- Schema versioning
schema_version (
  version TEXT PRIMARY KEY,
  description TEXT,
  applied_at DATETIME
)
```

## Target State

### Single Consolidated Migration File

**File:** `rust/crates/iqrah-storage/migrations_user/20241126000001_user_schema.sql` (RENAMED)

Create a single migration file that contains:

1. **Schema Version Table** (first, for tracking)
2. **Core FSRS Tables** (user_memory_states, session_state)
3. **Energy Propagation Tables** (propagation_events, propagation_details)
4. **App State Tables** (user_stats, app_settings)
5. **Scheduler v2 Bandit State** (user_bandit_state)
6. **Version Record** (single INSERT for v2.0.0)
7. **Optional: Test Data** (if not using Task 1.6 approach - see Implementation)

**Result:**
- Single migration file (~150 lines for schema + optional test data)
- Clear structure with section comments
- No redundancy, no duplicates
- All INTEGER content_key fields consistent
- Schema version recorded as 2.0.0 to match content.db

## Implementation Steps

### Step 1: Create Consolidated Migration File (1 hour)

**File:** Create `rust/crates/iqrah-storage/migrations_user/20241126000001_user_schema.sql`

Combine all schema definitions into single file with clear structure:

```sql
-- ============================================================================
-- User Database Schema v2.0.0
-- Date: 2025-11-26
-- Consolidated from 9 iterative migrations into single final schema
-- ============================================================================

-- ============================================================================
-- SCHEMA VERSION TRACKING
-- ============================================================================
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'User database schema v2 with integer IDs and Thompson Sampling bandit');

-- ============================================================================
-- FSRS + ENERGY TRACKING (Core learning state)
-- ============================================================================
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    content_key INTEGER NOT NULL,  -- i64 encoded node ID
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch milliseconds
    due_at INTEGER NOT NULL DEFAULT 0,          -- epoch milliseconds
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, content_key)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);
CREATE INDEX idx_ums_user_last ON user_memory_states(user_id, last_reviewed);

-- ============================================================================
-- SESSION STATE (Ephemeral - for session resume)
-- ============================================================================
CREATE TABLE session_state (
    content_key INTEGER NOT NULL PRIMARY KEY,  -- i64 encoded node ID
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- ============================================================================
-- ENERGY PROPAGATION TRACKING
-- ============================================================================
CREATE TABLE propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_content_key INTEGER NOT NULL,  -- i64 encoded source node ID
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);
CREATE INDEX idx_prop_events_source ON propagation_events(source_content_key);

CREATE TABLE propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_content_key INTEGER NOT NULL,  -- i64 encoded target node ID
    energy_change REAL NOT NULL,
    path TEXT,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_content_key);

-- ============================================================================
-- APP STATE (Settings and statistics)
-- ============================================================================
CREATE TABLE user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- ============================================================================
-- SCHEDULER V2 BANDIT OPTIMIZER STATE (Thompson Sampling)
-- ============================================================================
CREATE TABLE user_bandit_state (
    user_id TEXT NOT NULL,
    goal_group TEXT NOT NULL,        -- 'memorization', 'vocab', 'tajweed', etc.
    profile_name TEXT NOT NULL,      -- 'Balanced', 'FoundationHeavy', etc.
    successes REAL NOT NULL DEFAULT 1.0,  -- Beta distribution alpha
    failures REAL NOT NULL DEFAULT 1.0,   -- Beta distribution beta
    last_updated INTEGER NOT NULL DEFAULT 0,  -- Epoch milliseconds
    PRIMARY KEY (user_id, goal_group, profile_name)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_bandit_user_group ON user_bandit_state(user_id, goal_group);
CREATE INDEX idx_bandit_updated ON user_bandit_state(last_updated DESC);
```

**Checklist:**
- [ ] All schema definitions from 9 files consolidated
- [ ] All field types match final state (INTEGER content_key)
- [ ] All indexes recreated
- [ ] All constraints preserved
- [ ] Schema version set to 2.0.0
- [ ] Clear section comments
- [ ] No redundancy or duplication

### Step 2: Verify Migration Works (30 min)

Test the new consolidated migration:

```bash
# Clean build
cd rust
cargo clean

# Run tests - should use new consolidated migration
cargo test --all-features

# Verify schema is created correctly
# (Tests will implicitly verify this by querying the DB)
```

**Expected Results:**
- All tests pass
- Database initializes with clean single migration
- Schema version is 2.0.0
- All tables exist with correct columns and types

### Step 3: Delete Old Migration Files (10 min)

After verifying new migration works:

```bash
# Delete old migrations
rm rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql
rm rust/crates/iqrah-storage/migrations_user/20241116000002_initialize_settings.sql
rm rust/crates/iqrah-storage/migrations_user/20241117000001_content_keys.sql
rm rust/crates/iqrah-storage/migrations_user/20241117000002_scheduler_v2_bandit.sql
rm rust/crates/iqrah-storage/migrations_user/20241117000003_scheduler_v2_sample_user_data.sql
rm rust/crates/iqrah-storage/migrations_user/20241117000004_larger_test_user_data.sql
rm rust/crates/iqrah-storage/migrations_user/20241124000001_add_schema_version.sql
rm rust/crates/iqrah-storage/migrations_user/20241125000001_add_schema_version.sql
rm rust/crates/iqrah-storage/migrations_user/20241126000002_convert_content_key_to_integer.sql

# Verify only new file remains
ls -la rust/crates/iqrah-storage/migrations_user/
# Should show only: 20241126000001_user_schema.sql
```

### Step 4: Update Test Data Organization (1 hour - Optional Improvement)

If Task 1.6 has been implemented (test data separation):
- Remove any INSERT statements from the consolidated migration
- Test data should be seeded via `test_data.rs` module (per Task 1.6)

If Task 1.6 hasn't been done yet:
- Keep necessary test data INSERTs in the migration for now
- When Task 1.6 is implemented, extract test INSERTs to separate module

**Note:** This step depends on Task 1.6 status. See "Scope Limits" section.

### Step 5: Final Verification (1 hour)

Run complete CI validation:

```bash
cd rust

# Build with warnings as errors
RUSTFLAGS="-D warnings" cargo build --all-features

# Clippy
cargo clippy --all-features --all-targets -- -D warnings

# All tests
cargo test --all-features

# Formatting
cargo fmt --all -- --check
```

**Verification Queries** (optional manual verification):

```sql
-- Verify schema version
SELECT * FROM schema_version;  -- Should show 2.0.0

-- Verify table structure
PRAGMA table_info(user_memory_states);  -- content_key should be INTEGER
PRAGMA table_info(session_state);       -- content_key should be INTEGER
PRAGMA table_info(propagation_events);  -- source_content_key should be INTEGER

-- Verify indexes exist
SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='user_memory_states';
```

## Scope Limits & Safeguards

### ✅ MUST DO

- Consolidate all 9 migrations into 1 file
- Keep FINAL schema state (INTEGER content_key)
- Preserve all table structures, columns, and types exactly
- Preserve all indexes and constraints
- Update schema version to 2.0.0 (consistent with content.db)
- Ensure all tests pass without modification
- Clean single migration file ready for production

### ❌ DO NOT

- Change field names or types mid-consolidation
- Add new tables (consolidation only)
- Remove indexes or constraints
- Modify test behavior
- Create multiple migration files (goal is ONE file)
- Attempt to migrate actual user data (doesn't exist yet)
- Change content_key encoding or format

### ⚠️ Dependencies on Other Tasks

**Task 1.6 Status:**

- Task 1.6 now includes comprehensive schema redesign (v2.1) for content.db
- Task 1.7 focuses on user.db only - can be done in parallel
- Both tasks follow the same pattern: consolidate migrations, remove test data
- User DB test data approach can mirror content DB approach (separate test module if desired)

**Task 1.4 Status:**

- ✅ COMPLETE: i64 ID refactoring done. Consolidated migration should use INTEGER content_key throughout.

## Success Criteria

- [ ] Single consolidated migration file created: `20241126000001_user_schema.sql`
- [ ] All 9 old migration files deleted
- [ ] Migration file contains complete schema (all 8 tables with correct types)
- [ ] Schema version recorded as 2.0.0
- [ ] All tables use INTEGER for content_key fields (not TEXT)
- [ ] All indexes preserved and recreated
- [ ] All foreign keys and constraints preserved
- [ ] `cargo test --all-features` passes (all tests)
- [ ] `RUSTFLAGS="-D warnings" cargo build --all-features` passes (no warnings)
- [ ] `cargo clippy --all-features --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes (formatting clean)
- [ ] Simplified migrations_user/ directory (single file instead of 9)
- [ ] No test changes required (consolidation is transparent)

## Related Files

- **Old Migration Files**: `rust/crates/iqrah-storage/migrations_user/` (all 9 to be removed)
- **New Consolidated File**: `rust/crates/iqrah-storage/migrations_user/20241126000001_user_schema.sql` (to create)
- **User Repository Tests**: `rust/crates/iqrah-storage/src/user/scheduler_tests.rs` (must pass)
- **Storage Integration Tests**: `rust/crates/iqrah-storage/tests/integration_tests.rs` (must pass)

## Notes

### Why Consolidation Now?

The app is not yet in production. Real user data doesn't exist. This is the ideal time to consolidate iterative migrations into a clean final schema. Once users exist, we'd need to maintain backward compatibility with old migrations - much more complex.

### Migration Naming

The new file is named `20241126000001_user_schema.sql` (the date of consolidation). This is consistent with:
- Content DB: `20241126000001_unified_content_schema.sql` (same date consolidation)
- The sequential numbering matches sqlx's expectation of migration ordering

### Test Data Considerations

**After Task 1.6 is complete:**
- Test data will move from migrations to `test_data.rs` module
- Consolidated migration will contain ONLY schema (no INSERTs for sample data)
- Tests explicitly seed data via `init_test_content_db()` helper
- This maintains clean separation: schema in migrations, test data in test modules

**Current behavior (before Task 1.6):**
- If Task 1.6 not yet implemented, keep any test data INSERTs in consolidated migration
- This is acceptable as temporary state
- Task 1.6 can then extract them to separate module
