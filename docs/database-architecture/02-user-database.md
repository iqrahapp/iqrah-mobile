# User Database Architecture

**Related Question:** Q4 - How are user DB migrations handled?

## Overview

The User Database (user.db) is a **read-write SQLite database** created on the user's device containing:
- Learning progress (FSRS parameters + custom energy model)
- Energy propagation audit trail
- Session state (for resume functionality)
- User statistics and app settings

## Schema

**Location:** [rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql](../../rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql)

### Core Tables

#### 1. user_memory_states
Stores spaced repetition state for each node the user has reviewed.

```sql
CREATE TABLE IF NOT EXISTS user_memory_states (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER,
    due_at INTEGER,
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, node_id)
) WITHOUT ROWID, STRICT;

CREATE INDEX idx_user_memory_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_user_memory_node ON user_memory_states(node_id);
```

**Fields:**
- `user_id` - Multi-user support (currently always "default")
- `node_id` - References node in content.db (e.g., "WORD_INSTANCE:1:1:1")
- `stability` - FSRS parameter: how stable this memory is (higher = longer interval)
- `difficulty` - FSRS parameter: inherent difficulty of this item (0-10)
- `energy` - Custom model: propagated mastery level (0.0 - 1.0)
- `last_reviewed` - Unix timestamp
- `due_at` - Unix timestamp when next review is due
- `review_count` - Total number of reviews

**Model:** Hybrid approach combining:
- **FSRS (Free Spaced Repetition Scheduler)** - stability & difficulty
- **Custom energy propagation** - graph-based mastery spreading

#### 2. propagation_events
Audit log for energy propagation events.

```sql
CREATE TABLE IF NOT EXISTS propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_node_id TEXT NOT NULL,
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_propagation_timestamp ON propagation_events(event_timestamp);
```

**Purpose:** Track when energy propagation was triggered (e.g., after reviewing a word).

#### 3. propagation_details
Detailed breakdown of a propagation event.

```sql
CREATE TABLE IF NOT EXISTS propagation_details (
    event_id INTEGER NOT NULL,
    target_node_id TEXT NOT NULL,
    energy_change REAL NOT NULL,
    reason TEXT,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id)
) STRICT;

CREATE INDEX idx_propagation_target ON propagation_details(target_node_id);
```

**Purpose:** For each propagation event, record which nodes received energy and why.

**Example:**
```
Event #42 (source: WORD_INSTANCE:1:1:3)
  → target: VERSE:1:1, energy_change: +0.15, reason: "word to verse dependency"
  → target: WORD_INSTANCE:1:1:2, energy_change: +0.08, reason: "adjacent word knowledge edge"
```

#### 4. session_state
Ephemeral state for session resume functionality.

```sql
CREATE TABLE IF NOT EXISTS session_state (
    node_id TEXT PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT;
```

**Purpose:** When user exits mid-session, store the order of nodes so session can resume from where they left off.

**Lifecycle:**
- Created: When session starts
- Updated: As user progresses through exercises
- Cleared: When session completes

#### 5. user_stats
Generic key-value store for user statistics.

```sql
CREATE TABLE IF NOT EXISTS user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
```

**Example data:**
- `total_reviews_count` → "1247"
- `streak_days` → "23"
- `last_session_date` → "2024-11-16"

#### 6. app_settings
Generic key-value store for app configuration.

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
```

**Key setting:**
```sql
INSERT INTO app_settings (key, value) VALUES ('schema_version', '2');
```

This tracks the user DB schema version for future migrations.

## FSRS + Energy Hybrid Model

### Why Hybrid?

Traditional spaced repetition (like FSRS) optimizes **when** to review based on memory decay. Iqrah adds **graph-based energy propagation** to model how learning one item helps learn related items.

**Example:**
- User reviews "بِسْمِ" (bismillah) and gets it correct
- FSRS updates: stability increases, next review in 7 days
- Energy propagation:
  - Energy spreads to parent verse (learning word → helps verse memorization)
  - Energy spreads to adjacent words (context reinforcement)
  - Energy spreads via knowledge edges (translation understanding → memorization)

### FSRS Parameters

**Stability:** Time (in days) when recall probability drops to 90%.
- Higher stability = longer intervals between reviews
- Updated after each review based on grade (Again, Hard, Good, Easy)

**Difficulty:** Intrinsic difficulty of the material (0-10).
- Increases if user frequently struggles
- Affects future interval calculations

**Implementation:** Uses [fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) library.

**Code:** [learning_service.rs:10-150+](../../rust/crates/iqrah-core/src/services/learning_service.rs)

### Energy Propagation

**Energy:** Custom parameter (0.0 - 1.0) representing mastery level.
- 0.0 = Not learned
- 1.0 = Fully mastered
- Propagates through graph edges based on distribution parameters

**Propagation Algorithm:**
```rust
// Simplified from learning_service.rs
for edge in content_repo.get_edges_from(source_node_id) {
    let energy_change = calculate_energy_transfer(
        source_energy,
        edge.distribution_type,
        edge.param1,
        edge.param2
    );

    user_repo.update_energy(target_node_id, energy_change);
    user_repo.log_propagation(event_id, target_node_id, energy_change);
}
```

**Distribution Types:**
- **Constant:** Fixed energy transfer (e.g., 0.3 always)
- **Normal:** Sample from normal distribution (e.g., mean=0.4, std=0.1)
- **Beta:** Sample from beta distribution (for skewed relationships)

**See:** [04-database-interactions.md](04-database-interactions.md) for propagation flow details.

## Q4: User DB Migration Strategy

### Current Approach

**Method:** Standard [SQLx migrations](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#migrations)

**Migration Files:**
```
rust/crates/iqrah-storage/migrations_user/
├── 20241116000001_user_schema.sql          # Create all tables
└── 20241116000002_initialize_settings.sql  # Insert schema_version
```

**Execution:**
```rust
// rust/crates/iqrah-storage/src/user/mod.rs:10-19
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_path).await?;

    sqlx::migrate!("./migrations_user")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

**Tracking:**
1. **SQLx Internal:** `_sqlx_migrations` table tracks which files have run
2. **Explicit Version:** `app_settings` table has `schema_version` = '2'

**Example `_sqlx_migrations` table:**
```
version | description              | success | checksum   | execution_time
--------|--------------------------|---------|------------|---------------
1       | user_schema              | true    | [hash]     | 1700000001
2       | initialize_settings      | true    | [hash]     | 1700000002
```

### How to Add a Migration

**Example:** Add a new `user_achievements` table.

1. Create new migration file:
```bash
# Manually create (SQLx CLI not required)
touch migrations_user/20241116000003_add_achievements.sql
```

2. Write migration:
```sql
-- migrations_user/20241116000003_add_achievements.sql
CREATE TABLE user_achievements (
    achievement_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    unlocked_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES user_memory_states(user_id)
) STRICT;

-- Update schema version
UPDATE app_settings SET value = '3' WHERE key = 'schema_version';
```

3. SQLx automatically runs it on next `init_user_db()` call.

### Migration Safety

**Guarantees:**
- Migrations run in order (sorted by filename)
- Each migration runs exactly once (tracked by checksum)
- Transactional (rollback on error)

**Limitations:**
- **No down migrations** - SQLx doesn't support rollback scripts
- **No data migrations** - Complex data transformations require custom code

**User Data Preservation:**
- All migrations are **additive** (CREATE TABLE, ADD COLUMN)
- Never DROP or TRUNCATE user data tables
- Backward compatible schema changes preferred

### Schema Versioning

**Why both SQLx tracking AND app_settings version?**

1. **SQLx tracking:** For SQLx's internal state management
2. **app_settings version:** Application-level version for:
   - Feature flags (e.g., "achievements only available if schema >= 3")
   - Diagnostic logging
   - Explicit version checks in code

**Best Practice:**
```rust
// Check schema version before using new feature
let version: String = sqlx::query_scalar(
    "SELECT value FROM app_settings WHERE key = 'schema_version'"
).fetch_one(&pool).await?;

if version.parse::<i32>()? >= 3 {
    // Use achievements feature
}
```

## Multi-User Support

**Current Status:** Schema supports multi-user via `user_id` field.

**Implementation:** Currently hardcoded to "default":
```rust
// Example from code
let user_id = "default";
user_repo.get_memory_state(user_id, node_id).await?;
```

**Future:** Easy to extend to multiple profiles:
```sql
SELECT * FROM user_memory_states WHERE user_id = 'alice'
SELECT * FROM user_memory_states WHERE user_id = 'bob'
```

## Performance Considerations

### Indexes

**Critical indexes:**
```sql
CREATE INDEX idx_user_memory_due ON user_memory_states(user_id, due_at);
```
Used for: "Get all due items for review" query (very frequent).

```sql
CREATE INDEX idx_user_memory_node ON user_memory_states(node_id);
```
Used for: Energy propagation lookups.

**Query Performance:**
- `get_due_states()` - O(log n) via idx_user_memory_due
- `get_memory_state(node_id)` - O(log n) via idx_user_memory_node
- `log_propagation()` - O(1) append to propagation_details

### Database Size

**Growth rate:**
- Worst case: One row per node in content DB
- Typical: ~10-20% of nodes reviewed (most users don't review every word separately)
- Example: 80K nodes → ~8-16K memory states → ~1-2 MB user.db

**Cleanup:**
- Session state cleared after each session
- Propagation events could be archived/pruned (currently kept indefinitely)

## File Locations

**Rust Implementation:**
- Schema: [migrations_user/20241116000001_user_schema.sql](../../rust/crates/iqrah-storage/migrations_user/20241116000001_user_schema.sql)
- Repository: [src/user/repository.rs](../../rust/crates/iqrah-storage/src/user/repository.rs) (lines 7-199)
- Models: [src/user/models.rs](../../rust/crates/iqrah-storage/src/user/models.rs)

**Usage:**
- Learning Service: [services/learning_service.rs](../../rust/crates/iqrah-core/src/services/learning_service.rs)
- Session Service: [services/session_service.rs](../../rust/crates/iqrah-core/src/services/session_service.rs)

---

**Navigation:** [← Content Database](01-content-database.md) | [Next: Knowledge Graph →](03-knowledge-graph.md)
