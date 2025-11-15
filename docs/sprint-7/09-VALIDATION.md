# Step 9: Final Validation & Cleanup

## Goal
Verify that Sprint 7 is complete and all success criteria are met.

## Validation Checklist

### 1. Database Structure

#### Check content.db

```bash
sqlite3 ~/Documents/content.db "SELECT name FROM sqlite_master WHERE type='table'"
```

Expected tables:
- nodes
- edges
- node_metadata

```bash
sqlite3 ~/Documents/content.db "SELECT COUNT(*) FROM nodes"
```

Expected: Should match node count from old database (if migrated).

#### Check user.db

```bash
sqlite3 ~/Documents/user.db "SELECT name FROM sqlite_master WHERE type='table'"
```

Expected tables:
- user_memory_states
- propagation_events
- propagation_details
- session_state
- user_stats
- app_settings (from migration v2)
- _sqlx_migrations (migration tracking)

```bash
sqlite3 ~/Documents/user.db "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1"
```

Expected: `2` (both migrations ran)

```bash
sqlite3 ~/Documents/user.db "SELECT * FROM app_settings"
```

Expected output:
```
schema_version|2
migration_date|<timestamp>
```

### 2. Code Quality

#### No Compilation Warnings

```bash
cd /home/user/iqrah-mobile/rust
cargo clippy --workspace -- -D warnings
```

Expected: No warnings.

#### Code Formatting

```bash
cargo fmt --check
```

Expected: All code formatted.

### 3. Test Suite

#### Run All Tests

```bash
# Core tests
cargo test -p iqrah-core --lib
echo "✓ Core tests passed"

# Storage tests
cargo test -p iqrah-storage
echo "✓ Storage tests passed"

# Integration tests
cargo test --workspace --test '*'
echo "✓ Integration tests passed"
```

All tests should pass.

#### Flutter Tests

```bash
cd /home/user/iqrah-mobile
flutter test
echo "✓ Flutter tests passed"
```

### 4. Functional Validation

#### App Launch

```bash
flutter run
```

Expected: App launches without errors.

#### Database Initialization

Check logs for:
```
Migrating from old database...
Migration complete!
App initialized successfully
```

(Only on first run if old database existed)

#### Session Generation

From the app:
1. Navigate to Sessions tab
2. Click "Start Session"
3. Verify exercises load
4. Complete a review
5. Check that stats update

#### Propagation Logging

```bash
sqlite3 ~/Documents/user.db "SELECT COUNT(*) FROM propagation_events"
```

Should increase after completing reviews.

### 5. Data Integrity

#### Compare Old vs New (If Migrated)

```bash
# Old database (backed up)
OLD_NODES=$(sqlite3 ~/Documents/iqrah.db.backup "SELECT COUNT(*) FROM nodes")
NEW_NODES=$(sqlite3 ~/Documents/content.db "SELECT COUNT(*) FROM nodes")

echo "Old: $OLD_NODES nodes"
echo "New: $NEW_NODES nodes"

# Should match
if [ "$OLD_NODES" -eq "$NEW_NODES" ]; then
    echo "✓ Node migration verified"
else
    echo "✗ Node count mismatch!"
fi

# User states
OLD_STATES=$(sqlite3 ~/Documents/iqrah.db.backup "SELECT COUNT(*) FROM user_memory_states")
NEW_STATES=$(sqlite3 ~/Documents/user.db "SELECT COUNT(*) FROM user_memory_states")

echo "Old: $OLD_STATES states"
echo "New: $NEW_STATES states"

if [ "$OLD_STATES" -eq "$NEW_STATES" ]; then
    echo "✓ User state migration verified"
else
    echo "✗ User state count mismatch!"
fi
```

### 6. Performance Benchmarks

#### Session Generation Speed

Create a simple benchmark:

**File:** `rust/crates/iqrah-cli/src/main.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bench {
        #[arg(long)]
        content_db: String,
        #[arg(long)]
        user_db: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bench { content_db, user_db } => {
            use std::time::Instant;

            let content_pool = iqrah_storage::init_content_db(&content_db).await?;
            let user_pool = iqrah_storage::init_user_db(&user_db).await?;

            let content_repo = std::sync::Arc::new(
                iqrah_storage::SqliteContentRepository::new(content_pool)
            );
            let user_repo = std::sync::Arc::new(
                iqrah_storage::SqliteUserRepository::new(user_pool)
            );

            // Benchmark: Get 20 due items
            let start = Instant::now();
            let states = user_repo.get_due_states("default_user", chrono::Utc::now(), 20).await?;
            let duration = start.elapsed();

            println!("Session generation: {:?}", duration);
            println!("Items retrieved: {}", states.len());

            if duration.as_millis() < 50 {
                println!("✓ Performance target met (<50ms)");
            } else {
                println!("⚠ Performance slower than target");
            }
        }
    }

    Ok(())
}
```

Run benchmark:
```bash
cargo run --release --bin iqrah -- bench \
    --content-db ~/Documents/content.db \
    --user-db ~/Documents/user.db
```

Target: < 50ms

### 7. Documentation

#### Create MIGRATION.md

**File:** `docs/MIGRATION.md`

```markdown
# Database Migration Guide

## Overview

Sprint 7 introduced a two-database architecture:
- **content.db** - Immutable knowledge graph
- **user.db** - Mutable user progress

## Automatic Migration

On first launch, the app automatically detects the old `iqrah.db` and migrates:
1. Content data → content.db
2. User data → user.db
3. Old database → iqrah.db.backup

## Manual Migration

If needed, you can manually migrate using the CLI:

```bash
./target/release/iqrah migrate \
    --old-db ~/Documents/iqrah.db \
    --content-db ~/Documents/content.db \
    --user-db ~/Documents/user.db
```

## Rollback

To revert to the old database:

```bash
mv ~/Documents/iqrah.db.backup ~/Documents/iqrah.db
rm ~/Documents/content.db
rm ~/Documents/user.db
```

Then restart the app with the old version.

## Schema Versions

user.db uses SQLx migrations:
- v1: Initial schema
- v2: Added app_settings table

Check version:
```bash
sqlite3 ~/Documents/user.db "SELECT version FROM _sqlx_migrations"
```
```

#### Update README.md

Add section about Sprint 7:

```markdown
## Architecture (Sprint 7)

The app now uses a two-database architecture:

### content.db (Immutable)
- Qur'anic knowledge graph (nodes, edges)
- Metadata (Arabic text, translations)
- Replaced entirely on content updates

### user.db (Mutable)
- User progress (FSRS memory states)
- Energy propagation history
- Session state and statistics
- Never overwritten, only migrated

### Benefits
✓ Safe content updates
✓ Easy user data backups
✓ Clear separation of concerns
✓ Migration framework for schema evolution
```

## Final Cleanup

### Remove Old Code (if applicable)

If all tests pass and app works, we can remove the old monolithic files:

```bash
# These are now replaced by the new architecture
# Only delete if 100% confident everything works!

# Backup first
git tag pre-cleanup

# Remove old files (example - adjust based on actual structure)
# rm rust/src/sqlite_repo.rs  # Only if fully replaced
# rm rust/src/old_database.rs # Only if fully replaced
```

**IMPORTANT:** Only remove old code after thorough testing!

## Sprint 7 Acceptance Criteria

### Functional ✅
- [ ] Two separate databases created (content.db, user.db)
- [ ] All existing features work identically
- [ ] User data migrated successfully
- [ ] app_settings table created (proves migration v2 ran)
- [ ] PRAGMA user_version = 2 (or migration version = 2)

### Non-Functional ✅
- [ ] All tests pass (unit + integration)
- [ ] No compilation warnings
- [ ] Session generation works
- [ ] Review processing works
- [ ] Propagation logging works
- [ ] Stats update correctly

### Code Quality ✅
- [ ] Workspace structure implemented
- [ ] Repository pattern implemented
- [ ] Dependency injection (no global state in core)
- [ ] Migration framework functional
- [ ] Test coverage > 70%

### Documentation ✅
- [ ] MIGRATION.md created
- [ ] README.md updated
- [ ] Sprint 7 docs folder complete

## Sign-Off

Once all checkboxes above are complete:

```bash
# Tag the release
git tag sprint-7-complete
git push origin sprint-7-complete

# Create release notes
echo "Sprint 7: Stability & Foundation - COMPLETE" > RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "## Changes" >> RELEASE_NOTES.md
echo "- Two-database architecture (content.db + user.db)" >> RELEASE_NOTES.md
echo "- Migration framework for user.db" >> RELEASE_NOTES.md
echo "- Repository pattern with dependency injection" >> RELEASE_NOTES.md
echo "- Comprehensive test suite" >> RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
echo "## Breaking Changes" >> RELEASE_NOTES.md
echo "- Database structure changed (automatic migration on first run)" >> RELEASE_NOTES.md
```

## Known Issues / Tech Debt

Document any remaining issues:

**Example:**
- [ ] TODO: Optimize metadata queries (future: split into dedicated tables)
- [ ] TODO: Add CLI commands for debugging
- [ ] TODO: Property-based testing for energy propagation

## Next Steps (Sprint 8)

Sprint 7 foundation enables:
- Audio analysis features (requires content.db updates)
- Advanced exercise variants (requires testable scheduler)
- Multi-user support (requires user.db per user)

## Conclusion

Sprint 7 is complete when:
1. All tests pass ✓
2. All acceptance criteria met ✓
3. App runs without errors ✓
4. Data integrity verified ✓
5. Documentation updated ✓

**Status: [READY FOR PRODUCTION]**
