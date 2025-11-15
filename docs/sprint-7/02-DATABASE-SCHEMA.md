# Step 2: Database Schema Design

## Goal
Define the schemas for both `content.db` (immutable) and `user.db` (mutable).

## Schema Files Location
`rust/crates/iqrah-storage/migrations/`

## Content Database (content.db)

### Purpose
Immutable knowledge graph - replaced entirely on content updates.

### Schema File
**File:** `rust/crates/iqrah-storage/migrations/content_schema.sql`

```sql
-- Nodes: Entities in the knowledge graph
CREATE TABLE IF NOT EXISTS nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(node_type);

-- Edges: Relationships between nodes
CREATE TABLE IF NOT EXISTS edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    edge_type INTEGER NOT NULL,
    distribution_type INTEGER NOT NULL,
    param1 REAL NOT NULL DEFAULT 0.0,
    param2 REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES nodes(id),
    FOREIGN KEY (target_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id);
CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id);

-- Node Metadata (key-value for flexibility during migration)
CREATE TABLE IF NOT EXISTS node_metadata (
    node_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (node_id, key),
    FOREIGN KEY (node_id) REFERENCES nodes(id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_metadata_key ON node_metadata(key);
```

**Note:** We're keeping the existing metadata structure for now to simplify migration. Future optimization can split into dedicated tables (quran_text, translations, etc.).

## User Database (user.db)

### Purpose
Mutable user progress - sacred data that must never be lost.

### Migration Framework
Uses SQLx migrations with version tracking.

### Migration v1: Initial Schema
**File:** `rust/crates/iqrah-storage/migrations/00001_initial_schema.sql`

```sql
-- Migration v1: Initial user database schema

-- User Memory States (FSRS + Energy)
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,
    due_at INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, node_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);

-- Propagation Events
CREATE TABLE propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_node_id TEXT NOT NULL,
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);

-- Propagation Details
CREATE TABLE propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_node_id TEXT NOT NULL,
    energy_change REAL NOT NULL,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_node_id);

-- Session State (ephemeral - for session resume)
CREATE TABLE session_state (
    node_id TEXT NOT NULL PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- User Stats
CREATE TABLE user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
```

### Migration v2: Test Migration (App Settings)
**File:** `rust/crates/iqrah-storage/migrations/00002_app_settings.sql`

```sql
-- Migration v2: Add app settings table (proves migration harness works)

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- Insert default settings
INSERT INTO app_settings (key, value) VALUES
    ('schema_version', '2'),
    ('migration_date', strftime('%s', 'now'));
```

## Implementation Tasks

### Task 2.1: Create Migrations Directory

```bash
cd /home/user/iqrah-mobile/rust/crates/iqrah-storage
mkdir -p migrations
```

### Task 2.2: Create Schema Files

Create the three files above:
1. `migrations/content_schema.sql`
2. `migrations/00001_initial_schema.sql`
3. `migrations/00002_app_settings.sql`

### Task 2.3: Add Migration Metadata

SQLx requires a `.sqlx` directory for compile-time checks. We'll generate this during build.

Add to `iqrah-storage/Cargo.toml`:

```toml
[features]
# Enable compile-time query checking
sqlx-offline = []
```

## Database Initialization Logic

### Content DB Initialization

```rust
// In iqrah-storage/src/content/mod.rs
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run schema creation
    let schema = include_str!("../migrations/content_schema.sql");
    sqlx::raw_sql(schema).execute(&pool).await?;

    Ok(pool)
}
```

### User DB Initialization with Migrations

```rust
// In iqrah-storage/src/user/mod.rs
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub async fn init_user_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations (v1 and v2)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

## Validation

### Check Schema Files Exist

```bash
ls -la rust/crates/iqrah-storage/migrations/
```

Expected output:
```
content_schema.sql
00001_initial_schema.sql
00002_app_settings.sql
```

### Validate SQL Syntax

```bash
# Test content schema
sqlite3 :memory: < rust/crates/iqrah-storage/migrations/content_schema.sql

# Test user migrations
sqlite3 :memory: < rust/crates/iqrah-storage/migrations/00001_initial_schema.sql
sqlite3 :memory: < rust/crates/iqrah-storage/migrations/00002_app_settings.sql
```

No errors should be reported.

## Schema Comparison

### Old Schema (iqrah.db)
- `nodes`
- `edges`
- `node_metadata`
- `user_memory_states` ⚠️ User data
- `propagation_log` ⚠️ User data
- `session_state` ⚠️ User data

### New Schema (Split)

**content.db:**
- `nodes`
- `edges`
- `node_metadata`

**user.db:**
- `user_memory_states`
- `propagation_events`
- `propagation_details`
- `session_state`
- `user_stats`
- `app_settings` (NEW - from migration v2)

## Success Criteria

- [ ] 3 SQL files created
- [ ] SQL syntax validates
- [ ] Migration files numbered correctly (00001, 00002)
- [ ] Content schema matches old nodes/edges/metadata structure
- [ ] User schema includes all user-specific tables

## Next Step

Proceed to `03-IMPLEMENT-CORE.md`
