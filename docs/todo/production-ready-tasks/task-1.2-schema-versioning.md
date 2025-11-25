# Task 1.2: Add Schema Versioning & Migration System

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

Currently, there's no explicit version tracking in the databases. We also need to physically create the `nodes` table to support the new "Internal Ints, External Strings" architecture.

**Why This Matters:**
- **Versioning:** Prevents loading incompatible databases.
- **Migration:** The `nodes` table is required for the `NodeRegistry` (Task 1.3).

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

### 1. Schema Version Tables (Both DBs)
```sql
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 2. New Schema Tables (content.db only)
**`nodes` Table:**
```sql
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    ukey TEXT NOT NULL UNIQUE,
    node_type INTEGER NOT NULL
) STRICT;
```

**`knowledge_nodes` Table:**
```sql
CREATE TABLE knowledge_nodes (
    node_id INTEGER PRIMARY KEY,
    content_node_id INTEGER NOT NULL,
    axis INTEGER NOT NULL,
    FOREIGN KEY(node_id) REFERENCES nodes(id),
    FOREIGN KEY(content_node_id) REFERENCES nodes(id)
) STRICT;
```

### 3. Versioning Logic
(As previously defined: `get_schema_version`, `is_compatible`)

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
(Implement `get_schema_version` and `is_compatible` in `rust/crates/iqrah-storage/src/version.rs` as originally planned)

### Step 4: Update lib.rs (5 min)
(Export `version` module)

### Step 5: Add Version Check to Database Initialization (1 hour)
(Update `init_content_db` and `init_user_db` in `mod.rs` files as originally planned)

### Step 6: Add Error Type (30 min)
(Add `IncompatibleSchema` to `error.rs`)

### Step 7: Add Integration Test (1 hour)
(Create `tests/version_test.rs` as originally planned)

## Verification Plan

### Database Check
```bash
cd rust
sqlx migrate run --source crates/iqrah-storage/migrations_content --database-url sqlite://test.db
sqlite3 test.db ".schema nodes"
```
- [ ] `nodes` table exists with `id`, `ukey`, `node_type`
- [ ] `knowledge_nodes` table exists
- [ ] `schema_version` table exists

### Code Check
- [ ] `cargo test version` passes
- [ ] `init_content_db` fails if version is incompatible

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
- [ ] `schema_version` table populated with correct initial versions.
