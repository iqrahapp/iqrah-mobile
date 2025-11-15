# Step 5: Migration Harness Implementation

## Goal
Implement a robust migration system for `user.db` that:
1. Tracks schema version
2. Runs migrations automatically
3. Ensures idempotency (safe to run multiple times)
4. Uses SQLx's built-in migration support

## SQLx Migration System

SQLx provides built-in migration support using:
- Migration files: `migrations/*.sql`
- Version tracking in database
- Automatic up/down migration handling
- Compile-time verification

## Migration Files Already Created

From Step 2, we have:
- `migrations/00001_initial_schema.sql` - Creates base tables
- `migrations/00002_app_settings.sql` - Test migration (app_settings table)

## How SQLx Migrations Work

### File Naming Convention
```
<version>_<description>.sql

Examples:
00001_initial_schema.sql
00002_app_settings.sql
00003_add_review_history.sql
```

### Migration Metadata
SQLx creates a `_sqlx_migrations` table automatically to track:
- Which migrations have been applied
- When they were applied
- Checksums to detect tampering

### Running Migrations

**Programmatically (in Rust):**
```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

**CLI (for development):**
```bash
sqlx migrate run
```

## Integration with user.db Initialization

The migration harness is already integrated in `init_user_db()` function.

**File:** `rust/crates/iqrah-storage/src/user/mod.rs` (already created in Step 4)

```rust
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // This runs ALL pending migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

### What Happens on First Run?
1. Database doesn't exist → SQLx creates it
2. No migrations applied → SQLx runs migration 00001
3. Migration 00001 completes → Runs 00002
4. All migrations complete → Returns pool

### What Happens on Subsequent Runs?
1. Database exists → SQLx connects
2. Checks `_sqlx_migrations` table
3. Sees 00001, 00002 already applied → Skips them
4. Returns pool immediately

## Validation of Migrations

### Check Migration Status

Create a helper function to check migration status:

**File:** `rust/crates/iqrah-storage/src/user/mod.rs` (add this function)

```rust
use sqlx::Row;

/// Get current schema version
pub async fn get_schema_version(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get::<i64, _>("version") as i32).unwrap_or(0))
}

/// Check if a specific table exists
pub async fn table_exists(pool: &SqlitePool, table_name: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?"
    )
    .bind(table_name)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
```

## Testing Migrations

### Test 1: Fresh Database

```rust
#[tokio::test]
async fn test_fresh_database_runs_all_migrations() {
    use iqrah_storage::{init_user_db, user::{get_schema_version, table_exists}};

    // Create in-memory database
    let pool = init_user_db(":memory:").await.unwrap();

    // Check version (should be 2 after both migrations)
    let version = get_schema_version(&pool).await.unwrap();
    assert_eq!(version, 2);

    // Check migration 1 tables exist
    assert!(table_exists(&pool, "user_memory_states").await.unwrap());
    assert!(table_exists(&pool, "propagation_events").await.unwrap());
    assert!(table_exists(&pool, "session_state").await.unwrap());

    // Check migration 2 table exists
    assert!(table_exists(&pool, "app_settings").await.unwrap());

    // Check app_settings has data
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'schema_version'")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(row.is_some());
}
```

### Test 2: Idempotency

```rust
#[tokio::test]
async fn test_migrations_are_idempotent() {
    use tempfile::NamedTempFile;
    use iqrah_storage::init_user_db;

    // Create temp file
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();

    // Run migrations first time
    let pool1 = init_user_db(db_path).await.unwrap();
    pool1.close().await;

    // Run migrations second time (should not error)
    let pool2 = init_user_db(db_path).await.unwrap();
    pool2.close().await;

    // Should succeed without errors
}
```

## Future Migration Example

When you need to add a new table in the future:

### Step 1: Create Migration File

**File:** `migrations/00003_add_review_history.sql`

```sql
-- Migration v3: Add review history tracking

CREATE TABLE review_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL,
    grade INTEGER NOT NULL,
    duration_ms INTEGER,
    exercise_type TEXT
) STRICT;

CREATE INDEX idx_review_history_user_node ON review_history(user_id, node_id);
CREATE INDEX idx_review_history_timestamp ON review_history(reviewed_at DESC);
```

### Step 2: That's It!

Next time `init_user_db()` runs, SQLx will automatically:
1. Detect the new migration file
2. Run it if not already applied
3. Update the version tracking

## Rollback Strategy

SQLx supports down migrations (reverting changes):

**File:** `migrations/00003_add_review_history.down.sql`

```sql
-- Rollback migration v3

DROP TABLE IF EXISTS review_history;
```

To rollback:
```bash
sqlx migrate revert
```

## CLI Tool Integration

### Add Migration Check Command

**File:** `rust/crates/iqrah-cli/src/main.rs` (skeleton for future)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "iqrah")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check migration status
    MigrateStatus {
        #[arg(long)]
        user_db: String,
    },

    /// Run pending migrations
    MigrateRun {
        #[arg(long)]
        user_db: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::MigrateStatus { user_db } => {
            // Check status
            let pool = sqlx::SqlitePool::connect(&user_db).await?;
            let version = iqrah_storage::user::get_schema_version(&pool).await?;
            println!("Current schema version: {}", version);
        }

        Commands::MigrateRun { user_db } => {
            // Run migrations
            let pool = iqrah_storage::init_user_db(&user_db).await?;
            println!("Migrations complete!");
        }
    }

    Ok(())
}
```

## Success Criteria

- [ ] Migration files exist (00001, 00002)
- [ ] `init_user_db()` runs migrations automatically
- [ ] `_sqlx_migrations` table created
- [ ] `app_settings` table created (proves v2 ran)
- [ ] Schema version = 2
- [ ] Idempotent (safe to run multiple times)

## Validation Commands

```bash
# Create test database
cd /home/user/iqrah-mobile/rust/crates/iqrah-storage
cargo test --lib

# Check migration status
sqlite3 test_user.db "SELECT * FROM _sqlx_migrations"

# Check app_settings table exists
sqlite3 test_user.db "SELECT * FROM app_settings"
```

Expected output:
```
schema_version|2
migration_date|<timestamp>
```

## Next Step

Proceed to `06-DATA-MIGRATION.md`
