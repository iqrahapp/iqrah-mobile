# Task 1.2: Add Schema Versioning System

## Metadata
- **Priority:** P0 (Critical Foundation)
- **Estimated Effort:** 1 day
- **Dependencies:** None
- **Agent Type:** Implementation
- **Parallelizable:** Yes (with tasks 1.1, 1.3, 1.5)

## Goal

Implement schema version tracking in both content.db and user.db to enable safe migrations, compatibility checks, and prevent breaking changes when updating databases.

## Context

Currently, there's no explicit version tracking in the databases. SQLx migrations track which migration files have run (`_sqlx_migrations` table), but there's no application-level schema version that indicates compatibility.

**Why This Matters:**
- **Graph updates:** When user downloads new graph data (monthly), need to verify it's compatible with current content.db schema
- **App updates:** When app updates with new schema, need to detect and handle migration
- **User data safety:** Prevent loading incompatible databases (e.g., old app with new schema)
- **Debugging:** Easily identify which schema version is deployed

**Referenced in:** [docs/database-architecture/04-versioning-and-migration-strategy.md](/docs/database-architecture/04-versioning-and-migration-strategy.md) (documented but not implemented)

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

### Schema Version Tables

**content.db:**
```sql
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,  -- Format: "major.minor.patch"
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Initial version
INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'v2 purist schema with knowledge graph integration');
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

### Step 1: Create Migration for content.db Schema Version (30 min)

**File:** `rust/crates/iqrah-storage/migrations_content/20241124000001_add_schema_version.sql`

```sql
-- Add schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Record current schema version (v2 purist with scheduler v2)
INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'v2 purist schema with scheduler v2 and knowledge graph chapters 1-3');
```

### Step 2: Create Migration for user.db Schema Version (30 min)

**File:** `rust/crates/iqrah-storage/migrations_user/20241124000001_add_schema_version.sql`

```sql
-- Add schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Record current schema version
INSERT INTO schema_version (version, description)
VALUES ('1.0.0', 'Initial user schema with FSRS, propagation tracking, and scheduler v2 bandit');
```

### Step 3: Add Version Query Function (1 hour)

**File:** `rust/crates/iqrah-storage/src/version.rs` (new file)

```rust
use sqlx::{SqlitePool, Row};
use crate::error::Result;

/// Get the current schema version from the database
pub async fn get_schema_version(pool: &SqlitePool) -> Result<String> {
    let row = sqlx::query("SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1")
        .fetch_one(pool)
        .await?;

    Ok(row.try_get("version")?)
}

/// Check if database schema version is compatible with app version
pub fn is_compatible(db_version: &str, app_version: &str) -> bool {
    let db_parts = parse_version(db_version);
    let app_parts = parse_version(app_version);

    // Major version must match (breaking changes)
    if db_parts.0 != app_parts.0 {
        return false;
    }

    // Minor version: DB can be <= app version (backwards compatible)
    if db_parts.1 > app_parts.1 {
        return false;  // DB is newer than app
    }

    // Patch version doesn't affect compatibility
    true
}

fn parse_version(version: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    (
        parts.get(0).copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        // Same version
        assert!(is_compatible("2.0.0", "2.0.0"));

        // App newer (minor): Compatible
        assert!(is_compatible("2.0.0", "2.1.0"));

        // DB newer (minor): Incompatible
        assert!(!is_compatible("2.1.0", "2.0.0"));

        // Different major: Incompatible
        assert!(!is_compatible("1.0.0", "2.0.0"));
        assert!(!is_compatible("2.0.0", "1.0.0"));

        // Patch differences: Always compatible
        assert!(is_compatible("2.0.0", "2.0.5"));
        assert!(is_compatible("2.0.5", "2.0.0"));
    }
}
```

### Step 4: Update lib.rs to Export Version Module (5 min)

**File:** `rust/crates/iqrah-storage/src/lib.rs`

Add:
```rust
pub mod version;
```

### Step 5: Add Version Check to Database Initialization (1 hour)

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`

Update `init_content_db()`:
```rust
use crate::version::{get_schema_version, is_compatible};

pub async fn init_content_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    // Run migrations first
    sqlx::migrate!("./migrations_content")
        .run(&pool)
        .await?;

    // Verify schema version compatibility
    let db_version = get_schema_version(&pool).await?;
    let app_version = env!("CARGO_PKG_VERSION");

    if !is_compatible(&db_version, app_version) {
        return Err(StorageError::IncompatibleSchema {
            db_version,
            app_version: app_version.to_string(),
            message: "Content database schema is incompatible with this app version".to_string(),
        });
    }

    tracing::info!("Content DB initialized: schema v{}, app v{}", db_version, app_version);

    Ok(pool)
}
```

**File:** `rust/crates/iqrah-storage/src/user/mod.rs`

Similar update for `init_user_db()`.

### Step 6: Add Error Type for Schema Incompatibility (30 min)

**File:** `rust/crates/iqrah-storage/src/error.rs`

Add to `StorageError` enum:
```rust
#[error("Incompatible schema version: DB {db_version}, App {app_version} - {message}")]
IncompatibleSchema {
    db_version: String,
    app_version: String,
    message: String,
},
```

### Step 7: Add Integration Test (1 hour)

**File:** `rust/crates/iqrah-storage/tests/version_test.rs` (new file)

```rust
use iqrah_storage::version::{get_schema_version, is_compatible};
use iqrah_storage::content::init_content_db;
use iqrah_storage::user::init_user_db;
use tempfile::TempDir;

#[tokio::test]
async fn test_content_db_has_version() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("content.db");

    let pool = init_content_db(db_path.to_str().unwrap()).await.unwrap();
    let version = get_schema_version(&pool).await.unwrap();

    assert!(version.starts_with("2."), "Content DB should be version 2.x.x");
}

#[tokio::test]
async fn test_user_db_has_version() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("user.db");

    let pool = init_user_db(db_path.to_str().unwrap()).await.unwrap();
    let version = get_schema_version(&pool).await.unwrap();

    assert!(version.starts_with("1."), "User DB should be version 1.x.x");
}
```

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

- [ ] Migration files created for both databases
- [ ] `schema_version` table exists in content.db with version "2.0.0"
- [ ] `schema_version` table exists in user.db with version "1.0.0"
- [ ] `version.rs` module implements `get_schema_version()` and `is_compatible()`
- [ ] Unit tests pass (version compatibility logic)
- [ ] Integration tests pass (version initialization)
- [ ] CLI test shows version tables populated correctly
- [ ] All CI checks pass (build, clippy, test, fmt)
- [ ] Error handling for incompatible versions works

## Related Files

**Create These Files:**
- `/rust/crates/iqrah-storage/migrations_content/20241124000001_add_schema_version.sql`
- `/rust/crates/iqrah-storage/migrations_user/20241124000001_add_schema_version.sql`
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
