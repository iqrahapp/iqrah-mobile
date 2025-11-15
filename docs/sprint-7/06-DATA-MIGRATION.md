# Step 6: Data Migration from Old Database

## Goal
Migrate existing user data from the old single `iqrah.db` to the new two-database architecture.

## Migration Strategy

### What Gets Migrated

**From `iqrah.db` to `content.db`:**
- `nodes` table (all rows)
- `edges` table (all rows)
- `node_metadata` table (all rows)

**From `iqrah.db` to `user.db`:**
- `user_memory_states` table (user progress)
- `propagation_log` table → split into `propagation_events` + `propagation_details`
- `session_state` table (if exists)

### When Migration Happens

Migration occurs automatically on first run of the new app:
1. Check if old `iqrah.db` exists
2. Check if new `user.db` doesn't exist
3. If both conditions true → run one-time migration
4. Mark migration as complete

## Implementation

### Task 6.1: Migration Module

**File:** `rust/crates/iqrah-storage/src/migrations/mod.rs`

```rust
use sqlx::{SqlitePool, Row};
use anyhow::Result;
use std::path::Path;

/// Check if old database exists
pub fn old_db_exists(old_db_path: &str) -> bool {
    Path::new(old_db_path).exists()
}

/// Migrate data from old single database to new two-database architecture
pub async fn migrate_from_old_db(
    old_db_path: &str,
    content_pool: &SqlitePool,
    user_pool: &SqlitePool,
) -> Result<()> {
    println!("Starting migration from {}", old_db_path);

    // Connect to old database
    let old_pool = SqlitePool::connect(&format!("sqlite://{}", old_db_path)).await?;

    // 1. Migrate content data (nodes, edges, metadata)
    migrate_content_data(&old_pool, content_pool).await?;

    // 2. Migrate user data (memory states, propagation log)
    migrate_user_data(&old_pool, user_pool).await?;

    println!("Migration complete!");

    Ok(())
}

/// Migrate content tables
async fn migrate_content_data(old_pool: &SqlitePool, content_pool: &SqlitePool) -> Result<()> {
    println!("  Migrating nodes...");

    // Copy nodes
    let nodes = sqlx::query("SELECT id, node_type, created_at FROM nodes")
        .fetch_all(old_pool)
        .await?;

    for node in &nodes {
        let id: String = node.get("id");
        let node_type: String = node.get("node_type");
        let created_at: i64 = node.get("created_at");

        sqlx::query(
            "INSERT OR IGNORE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)"
        )
        .bind(&id)
        .bind(&node_type)
        .bind(created_at)
        .execute(content_pool)
        .await?;
    }

    println!("    {} nodes migrated", nodes.len());

    println!("  Migrating edges...");

    // Copy edges
    let edges = sqlx::query(
        "SELECT source_id, target_id, edge_type, distribution_type, param1, param2 FROM edges"
    )
    .fetch_all(old_pool)
    .await?;

    for edge in &edges {
        sqlx::query(
            "INSERT OR IGNORE INTO edges
             (source_id, target_id, edge_type, distribution_type, param1, param2)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(edge.get::<String, _>("source_id"))
        .bind(edge.get::<String, _>("target_id"))
        .bind(edge.get::<i32, _>("edge_type"))
        .bind(edge.get::<i32, _>("distribution_type"))
        .bind(edge.get::<f64, _>("param1"))
        .bind(edge.get::<f64, _>("param2"))
        .execute(content_pool)
        .await?;
    }

    println!("    {} edges migrated", edges.len());

    println!("  Migrating metadata...");

    // Copy metadata
    let metadata = sqlx::query("SELECT node_id, key, value FROM node_metadata")
        .fetch_all(old_pool)
        .await?;

    for meta in &metadata {
        sqlx::query(
            "INSERT OR IGNORE INTO node_metadata (node_id, key, value) VALUES (?, ?, ?)"
        )
        .bind(meta.get::<String, _>("node_id"))
        .bind(meta.get::<String, _>("key"))
        .bind(meta.get::<String, _>("value"))
        .execute(content_pool)
        .await?;
    }

    println!("    {} metadata entries migrated", metadata.len());

    Ok(())
}

/// Migrate user tables
async fn migrate_user_data(old_pool: &SqlitePool, user_pool: &SqlitePool) -> Result<()> {
    println!("  Migrating user memory states...");

    // Copy user_memory_states
    let states = sqlx::query(
        "SELECT user_id, node_id, stability, difficulty, energy,
                last_reviewed, due_at, review_count
         FROM user_memory_states"
    )
    .fetch_all(old_pool)
    .await?;

    for state in &states {
        sqlx::query(
            "INSERT OR IGNORE INTO user_memory_states
             (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(state.get::<String, _>("user_id"))
        .bind(state.get::<String, _>("node_id"))
        .bind(state.get::<f64, _>("stability"))
        .bind(state.get::<f64, _>("difficulty"))
        .bind(state.get::<f64, _>("energy"))
        .bind(state.get::<i64, _>("last_reviewed"))
        .bind(state.get::<i64, _>("due_at"))
        .bind(state.get::<i64, _>("review_count"))
        .execute(user_pool)
        .await?;
    }

    println!("    {} memory states migrated", states.len());

    // Migrate propagation_log if it exists
    let table_exists = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='propagation_log'"
    )
    .fetch_optional(old_pool)
    .await?;

    if table_exists.is_some() {
        println!("  Migrating propagation log...");

        // Old schema: propagation_log(id, source_node_id, target_node_id, energy_change, reason, timestamp)
        // New schema: propagation_events + propagation_details

        let logs = sqlx::query(
            "SELECT id, source_node_id, target_node_id, energy_change, reason, timestamp
             FROM propagation_log
             ORDER BY timestamp, id"
        )
        .fetch_all(old_pool)
        .await?;

        // Group by source_node_id + timestamp to create events
        use std::collections::HashMap;
        let mut events: HashMap<(String, i64), Vec<(String, f64, String)>> = HashMap::new();

        for log in &logs {
            let source: String = log.get("source_node_id");
            let target: String = log.get("target_node_id");
            let timestamp: i64 = log.get("timestamp");
            let energy_change: f64 = log.get("energy_change");
            let reason: String = log.get("reason");

            events
                .entry((source.clone(), timestamp))
                .or_insert_with(Vec::new)
                .push((target, energy_change, reason));
        }

        for ((source_node_id, timestamp), details) in events {
            // Insert event
            let result = sqlx::query(
                "INSERT INTO propagation_events (source_node_id, event_timestamp) VALUES (?, ?)"
            )
            .bind(&source_node_id)
            .bind(timestamp)
            .execute(user_pool)
            .await?;

            let event_id = result.last_insert_rowid();

            // Insert details
            for (target, energy_change, reason) in details {
                sqlx::query(
                    "INSERT INTO propagation_details
                     (event_id, target_node_id, energy_change, reason)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(event_id)
                .bind(&target)
                .bind(energy_change)
                .bind(&reason)
                .execute(user_pool)
                .await?;
            }
        }

        println!("    Propagation log migrated");
    }

    // Migrate session_state if exists
    let session_exists = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='session_state'"
    )
    .fetch_optional(old_pool)
    .await?;

    if session_exists.is_some() {
        println!("  Migrating session state...");

        let sessions = sqlx::query("SELECT node_id, session_order FROM session_state")
            .fetch_all(old_pool)
            .await?;

        for session in &sessions {
            sqlx::query(
                "INSERT OR IGNORE INTO session_state (node_id, session_order) VALUES (?, ?)"
            )
            .bind(session.get::<String, _>("node_id"))
            .bind(session.get::<i64, _>("session_order"))
            .execute(user_pool)
            .await?;
        }

        println!("    {} session items migrated", sessions.len());
    }

    Ok(())
}

/// Mark migration as complete by creating a marker file
pub fn mark_migration_complete(marker_path: &str) -> Result<()> {
    use std::fs;
    fs::write(marker_path, "migrated")?;
    Ok(())
}

/// Check if migration has been completed
pub fn is_migration_complete(marker_path: &str) -> bool {
    Path::new(marker_path).exists()
}
```

### Task 6.2: Update Storage lib.rs

**File:** `rust/crates/iqrah-storage/src/lib.rs`

```rust
pub mod content;
pub mod user;
pub mod migrations;

pub use content::{SqliteContentRepository, init_content_db};
pub use user::{SqliteUserRepository, init_user_db};
pub use migrations::{migrate_from_old_db, old_db_exists, is_migration_complete, mark_migration_complete};
```

### Task 6.3: Integration with App Initialization

The migration will be triggered from the API layer during initialization.

**Pseudocode for `iqrah-api/src/api.rs`:**

```rust
pub async fn init_app(
    content_db_path: String,
    user_db_path: String,
    old_db_path: Option<String>,
) -> Result<String> {
    // Initialize content.db
    let content_pool = init_content_db(&content_db_path).await?;

    // Initialize user.db (runs migrations)
    let user_pool = init_user_db(&user_db_path).await?;

    // Check if we need to migrate from old database
    if let Some(old_path) = old_db_path {
        let migration_marker = format!("{}.migrated", old_path);

        if old_db_exists(&old_path) && !is_migration_complete(&migration_marker) {
            println!("Detected old database, running one-time migration...");

            migrate_from_old_db(&old_path, &content_pool, &user_pool).await?;

            mark_migration_complete(&migration_marker)?;

            println!("Migration complete! Old database backed up.");

            // Optionally rename old database
            std::fs::rename(&old_path, format!("{}.backup", old_path))?;
        }
    }

    // ... rest of initialization

    Ok("Initialized successfully".to_string())
}
```

## Testing Migration

### Test with Sample Data

**File:** `rust/crates/iqrah-storage/tests/migration_tests.rs`

```rust
use iqrah_storage::{init_content_db, init_user_db, migrate_from_old_db};
use sqlx::{SqlitePool, Row};
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_migration_from_old_db() {
    // 1. Create old database with sample data
    let old_db_file = NamedTempFile::new().unwrap();
    let old_db_path = old_db_file.path().to_str().unwrap();

    let old_pool = SqlitePool::connect(&format!("sqlite://{}", old_db_path))
        .await
        .unwrap();

    // Create old schema
    sqlx::query(
        "CREATE TABLE nodes (id TEXT PRIMARY KEY, node_type TEXT, created_at INTEGER)"
    )
    .execute(&old_pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE user_memory_states (
            user_id TEXT, node_id TEXT, stability REAL, difficulty REAL,
            energy REAL, last_reviewed INTEGER, due_at INTEGER, review_count INTEGER,
            PRIMARY KEY (user_id, node_id)
        )"
    )
    .execute(&old_pool)
    .await
    .unwrap();

    // Insert test data
    sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
        .execute(&old_pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO user_memory_states VALUES ('user1', 'node1', 1.0, 5.0, 0.5, 0, 0, 1)"
    )
    .execute(&old_pool)
    .await
    .unwrap();

    old_pool.close().await;

    // 2. Create new databases
    let content_pool = init_content_db(":memory:").await.unwrap();
    let user_pool = init_user_db(":memory:").await.unwrap();

    // 3. Run migration
    migrate_from_old_db(old_db_path, &content_pool, &user_pool)
        .await
        .unwrap();

    // 4. Verify data migrated
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&content_pool)
        .await
        .unwrap();
    assert_eq!(node_count, 1);

    let state_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_memory_states")
        .fetch_one(&user_pool)
        .await
        .unwrap();
    assert_eq!(state_count, 1);

    // Verify data integrity
    let state = sqlx::query("SELECT energy FROM user_memory_states WHERE node_id = 'node1'")
        .fetch_one(&user_pool)
        .await
        .unwrap();

    let energy: f64 = state.get("energy");
    assert_eq!(energy, 0.5);
}
```

## Manual Migration Process

For development/testing, you can manually run migration:

```bash
# Build CLI tool
cd /home/user/iqrah-mobile/rust
cargo build --bin iqrah --release

# Run migration
./target/release/iqrah migrate \
    --old-db ~/Documents/iqrah.db \
    --content-db ~/Documents/content.db \
    --user-db ~/Documents/user.db

# Verify
sqlite3 ~/Documents/content.db "SELECT COUNT(*) FROM nodes"
sqlite3 ~/Documents/user.db "SELECT COUNT(*) FROM user_memory_states"
```

## Rollback Plan

If migration fails:
1. Old database is never deleted (only renamed to `.backup`)
2. Can revert by renaming back:
   ```bash
   mv ~/Documents/iqrah.db.backup ~/Documents/iqrah.db
   rm ~/Documents/content.db
   rm ~/Documents/user.db
   ```

## Success Criteria

- [ ] Migration function compiles
- [ ] Can migrate nodes, edges, metadata to content.db
- [ ] Can migrate user_memory_states to user.db
- [ ] Can migrate propagation_log to new schema
- [ ] Migration is idempotent (safe to run twice)
- [ ] Old database is backed up
- [ ] Migration marker prevents re-running

## Validation

```bash
# Run migration tests
cargo test -p iqrah-storage migration_tests

# Manual verification
sqlite3 content.db "SELECT COUNT(*) FROM nodes"
sqlite3 user.db "SELECT COUNT(*) FROM user_memory_states"
sqlite3 user.db "SELECT * FROM app_settings WHERE key = 'schema_version'"
```

Expected: All data migrated successfully, no data loss.

## Next Step

Proceed to `07-UPDATE-API.md`
