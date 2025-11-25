# Task 1.2: Schema Migration & Versioning

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None
- **Agent Type:** Implementation
- **Parallelizable:** No (Blocker for 1.3, 1.4, 2.1)

## Goal

1.  Implement **Schema Versioning** tables in both databases.
2.  **Migrate `content.db`** to the new Integer-Based Architecture by creating the `nodes` and `knowledge_nodes` tables.
3.  Implement version compatibility checks in Rust.

## Context

The architecture is shifting to "Internal Ints, External Strings". This requires a physical `nodes` table to map between them.
We also need version tracking to ensure safe migrations.

**The Missing Piece:**
Currently, `content.db` has `edges` but no `nodes` table. We need to create it to support the `NodeRegistry`.

## Current State

**Migrations:**
- `rust/crates/iqrah-storage/migrations_content/` - 4 migration files
- `rust/crates/iqrah-storage/migrations_user/` - 6 migration files
- SQLx auto-generates `_sqlx_migrations` table with:
  - `version BIGINT` (migration file timestamp)
  - `description TEXT`
  - `installed_on TIMESTAMP`

**No Application Version:**
- No `schema_version` table
- No version compatibility checks in `init_content_db()` or `init_user_db()`
- Can't query "what version is this database?"

## Target State

### 1. New Tables (content.db)

**`nodes` Table:**
The central registry for all graph nodes.
```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,      -- Internal Integer ID (volatile)
    ukey TEXT NOT NULL UNIQUE,   -- External String ID (stable)
    node_type INTEGER NOT NULL   -- Enum mapping
) STRICT;
```

**`knowledge_nodes` Table:**
Metadata for knowledge nodes (derived from content).
```sql
CREATE TABLE knowledge_nodes (
    node_id INTEGER PRIMARY KEY,
    content_node_id INTEGER NOT NULL,
    axis INTEGER NOT NULL,       -- Enum: Memorization, Translation, etc.
    FOREIGN KEY(node_id) REFERENCES nodes(id),
    FOREIGN KEY(content_node_id) REFERENCES nodes(id)
) STRICT;
```

**`schema_version` Table:**
Tracks the database version.
```sql
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**user.db:**
```sql
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Initial version
INSERT INTO schema_version (version, description)
VALUES ('1.0.0', 'Initial user schema with FSRS and scheduler v2');
```

### 2. Versioning Logic
(Same as before: `get_schema_version`, `is_compatible`)

### Versioning Strategy

**Semantic Versioning:** `major.minor.patch`

- **Major (X.0.0):** Breaking changes
  - Node ID format changes
  - Column removals
  - Data type changes requiring migration
  - Example: Changing from string IDs to integer IDs

- **Minor (2.X.0):** Backwards-compatible additions
  - New tables (e.g., audio tables)
  - New columns with defaults
  - New node IDs added to graph
  - Example: Adding chapters 4-114 to graph

- **Patch (2.0.X):** Bug fixes, data corrections
  - PageRank score recalculations
  - Edge weight adjustments
  - No schema changes
  - Example: Fixing incorrect edge distribution

### Version Compatibility Checks

**In Rust `init_content_db()` and `init_user_db()`:**
```rust
async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    // Run migrations
    sqlx::migrate!("./migrations_content").run(&pool).await?;

    // Verify schema version compatibility
    let db_version = get_schema_version(&pool).await?;
    let app_version = env!("CARGO_PKG_VERSION"); // From Cargo.toml

    if !is_compatible(&db_version, app_version) {
        return Err(Error::IncompatibleSchemaVersion {
            db_version,
            app_version: app_version.to_string(),
        });
    }

    Ok(pool)
}
```

## Implementation Steps

### Step 1: Create Migration for content.db (1 hour)

**File:** `rust/crates/iqrah-storage/migrations_content/20241125000001_migrate_to_v2_schema.sql`

```sql
-- 1. Schema Versioning
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'v2 schema: nodes, knowledge_nodes, and integer IDs');

-- 2. Nodes Table (The Registry)
CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_nodes_ukey ON nodes(ukey);

-- 3. Knowledge Nodes Table
CREATE TABLE IF NOT EXISTS knowledge_nodes (
    node_id INTEGER PRIMARY KEY,
    content_node_id INTEGER NOT NULL,
    axis INTEGER NOT NULL,
    FOREIGN KEY(node_id) REFERENCES nodes(id),
    FOREIGN KEY(content_node_id) REFERENCES nodes(id)
) STRICT;

CREATE INDEX idx_knowledge_nodes_content ON knowledge_nodes(content_node_id);
```

### Step 2: Create Migration for user.db (30 min)

**File:** `rust/crates/iqrah-storage/migrations_user/20241125000001_add_schema_version.sql`

```sql
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO schema_version (version, description)
VALUES ('1.0.0', 'Initial user schema');
```

### Step 3: Implement Version Checking in Rust (1 hour)

**File:** `rust/crates/iqrah-storage/src/version.rs`
(Implement `get_schema_version` and `is_compatible` as previously defined)

**File:** `rust/crates/iqrah-storage/src/lib.rs`
(Export `version` module)

### Step 4: Update DB Init Logic (1 hour)

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`
**File:** `rust/crates/iqrah-storage/src/user/mod.rs`
(Add version checks to `init_content_db` and `init_user_db`)

## Verification Plan

### Unit Tests

- [ ] `cargo test version` - Version parsing and compatibility logic
- [ ] Test cases:
  - Same version → compatible
  - App newer minor version → compatible
  - DB newer minor version → incompatible
  - Different major version → incompatible
  - Patch differences → always compatible

### Integration Tests

- [ ] `cargo test test_content_db_has_version` - Content DB initialized with version
- [ ] `cargo test test_user_db_has_version` - User DB initialized with version
- [ ] Both tests pass after migrations run

### Database Check
```bash
cd rust
sqlx migrate run --source crates/iqrah-storage/migrations_content --database-url sqlite://test.db
sqlite3 test.db ".schema nodes"
```
- [ ] `nodes` table exists with `id`, `ukey`, `node_type`
- [ ] `knowledge_nodes` table exists
- [ ] `schema_version` table exists

### CLI Tests

```bash
# Initialize databases
cd rust
cargo run --bin iqrah-cli -- init

# Query versions
sqlite3 /tmp/test_content.db "SELECT * FROM schema_version;"
# Should output: 2.0.0|v2 purist schema...|<timestamp>

sqlite3 /tmp/test_user.db "SELECT * FROM schema_version;"
# Should output: 1.0.0|Initial user schema...|<timestamp>
```

### Manual Verification

- [ ] Run `cargo build` - Compiles without errors
- [ ] Run `cargo clippy -- -D warnings` - No warnings
- [ ] Run `cargo fmt --all -- --check` - Formatted correctly
- [ ] Migrations apply successfully on fresh databases
- [ ] Version queries return expected values

## Scope Limits & Safeguards

### ✅ MUST DO

- Add `schema_version` table to both databases
- Implement version compatibility checking
- Add unit tests for compatibility logic
- Add integration tests for version initialization
- Handle incompatible versions gracefully (return error, don't crash)

### ❌ DO NOT

- Change existing migration files (only add new ones)
- Modify existing tables (this is additive only)
- Implement automatic migration logic (out of scope)
- Add version bump automation (manual for now)
- Touch Flutter/UI code (Rust only)

### ⚠️ If Uncertain

- If migration fails during testing → check SQL syntax
- If version parsing fails → verify semantic versioning format (X.Y.Z)
- If compatibility logic seems complex → start with strict major version matching only
- If tests fail → check that migrations created the table successfully

## Success Criteria
- [ ] All tables (`nodes`, `knowledge_nodes`, `schema_version`) created via migration.
- [ ] Version checking logic implemented and tested.

## Related Files

**Create These Files:**
- `/rust/crates/iqrah-storage/migrations_content/20241125000001_migrate_to_v2_schema.sql`
- `/rust/crates/iqrah-storage/migrations_user/20241125000001_add_schema_version.sql`
- `/rust/crates/iqrah-storage/src/version.rs`
- `/rust/crates/iqrah-storage/tests/version_test.rs`

**Modify These Files:**
- `/rust/crates/iqrah-storage/src/lib.rs` - Export version module
- `/rust/crates/iqrah-storage/src/content/mod.rs` - Add version check to init
- `/rust/crates/iqrah-storage/src/user/mod.rs` - Add version check to init
- `/rust/crates/iqrah-storage/src/error.rs` - Add IncompatibleSchema error

**Reference Documentation:**
- `/docs/database-architecture/04-versioning-and-migration-strategy.md` - Original design spec

## Notes

**Current Versions:**
- Content DB: 2.0.0 (v2 purist schema from Nov 2024)
- User DB: 1.0.0 (initial schema from Nov 2024)

**Future Version Bumps:**
- When adding chapters 4-114 → content DB 2.1.0
- When adding audio tables → content DB 2.2.0
- When changing node ID format → content DB 3.0.0 (breaking)

**Why This Task First:**
Versioning is foundational. Later tasks (especially graph updates in Task 3.3) will rely on version checking to prevent incompatible data from being loaded.
