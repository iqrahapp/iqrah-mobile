# Step 4: Implement Storage Layer

## Goal
Implement repository traits using SQLx for both content.db and user.db.

## Implementation Structure

```
iqrah-storage/src/
├── lib.rs
├── content/
│   ├── mod.rs
│   └── repository.rs
├── user/
│   ├── mod.rs
│   └── repository.rs
└── migrations/
    ├── content_schema.sql
    ├── 00001_initial_schema.sql
    └── 00002_app_settings.sql
```

## Task 4.1: Content Repository Implementation

**File:** `rust/crates/iqrah-storage/src/content/repository.rs`

```rust
use sqlx::{SqlitePool, Row};
use async_trait::async_trait;
use std::collections::HashMap;
use iqrah_core::{ContentRepository, Node, NodeType, Edge, EdgeType, DistributionType};

pub struct SqliteContentRepository {
    pool: SqlitePool,
}

impl SqliteContentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContentRepository for SqliteContentRepository {
    async fn get_node(&self, node_id: &str) -> anyhow::Result<Option<Node>> {
        let row = sqlx::query(
            "SELECT id, node_type FROM nodes WHERE id = ?"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            Node {
                id: r.get("id"),
                node_type: NodeType::from(r.get::<String, _>("node_type")),
            }
        }))
    }

    async fn get_edges_from(&self, source_id: &str) -> anyhow::Result<Vec<Edge>> {
        let rows = sqlx::query(
            "SELECT source_id, target_id, edge_type, distribution_type, param1, param2
             FROM edges
             WHERE source_id = ?"
        )
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            let edge_type_val: i32 = r.get("edge_type");
            let dist_type_val: i32 = r.get("distribution_type");

            Edge {
                source_id: r.get("source_id"),
                target_id: r.get("target_id"),
                edge_type: if edge_type_val == 0 { EdgeType::Dependency } else { EdgeType::Knowledge },
                distribution_type: match dist_type_val {
                    0 => DistributionType::Const,
                    1 => DistributionType::Normal,
                    _ => DistributionType::Beta,
                },
                param1: r.get("param1"),
                param2: r.get("param2"),
            }
        }).collect())
    }

    async fn get_metadata(&self, node_id: &str, key: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            "SELECT value FROM node_metadata WHERE node_id = ? AND key = ?"
        )
        .bind(node_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("value")))
    }

    async fn get_all_metadata(&self, node_id: &str) -> anyhow::Result<HashMap<String, String>> {
        let rows = sqlx::query(
            "SELECT key, value FROM node_metadata WHERE node_id = ?"
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        let mut metadata = HashMap::new();
        for row in rows {
            metadata.insert(row.get("key"), row.get("value"));
        }

        Ok(metadata)
    }

    async fn node_exists(&self, node_id: &str) -> anyhow::Result<bool> {
        let row = sqlx::query(
            "SELECT 1 FROM nodes WHERE id = ? LIMIT 1"
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }
}
```

**File:** `rust/crates/iqrah-storage/src/content/mod.rs`

```rust
pub mod repository;

pub use repository::SqliteContentRepository;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

/// Initialize content database
pub async fn init_content_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run schema creation
    let schema = include_str!("../migrations/content_schema.sql");
    sqlx::raw_sql(schema).execute(&pool).await?;

    Ok(pool)
}
```

## Task 4.2: User Repository Implementation

**File:** `rust/crates/iqrah-storage/src/user/repository.rs`

```rust
use sqlx::{SqlitePool, Row};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iqrah_core::{UserRepository, MemoryState, PropagationEvent, PropagationDetail};

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn get_memory_state(&self, user_id: &str, node_id: &str) -> anyhow::Result<Option<MemoryState>> {
        let row = sqlx::query(
            "SELECT user_id, node_id, stability, difficulty, energy,
                    last_reviewed, due_at, review_count
             FROM user_memory_states
             WHERE user_id = ? AND node_id = ?"
        )
        .bind(user_id)
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            MemoryState {
                user_id: r.get("user_id"),
                node_id: r.get("node_id"),
                stability: r.get("stability"),
                difficulty: r.get("difficulty"),
                energy: r.get("energy"),
                last_reviewed: DateTime::from_timestamp_millis(r.get("last_reviewed"))
                    .unwrap_or_else(|| Utc::now()),
                due_at: DateTime::from_timestamp_millis(r.get("due_at"))
                    .unwrap_or_else(|| Utc::now()),
                review_count: r.get::<i64, _>("review_count") as u32,
            }
        }))
    }

    async fn save_memory_state(&self, state: &MemoryState) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO user_memory_states
             (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(user_id, node_id) DO UPDATE SET
                stability = excluded.stability,
                difficulty = excluded.difficulty,
                energy = excluded.energy,
                last_reviewed = excluded.last_reviewed,
                due_at = excluded.due_at,
                review_count = excluded.review_count"
        )
        .bind(&state.user_id)
        .bind(&state.node_id)
        .bind(state.stability)
        .bind(state.difficulty)
        .bind(state.energy)
        .bind(state.last_reviewed.timestamp_millis())
        .bind(state.due_at.timestamp_millis())
        .bind(state.review_count as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_due_states(&self, user_id: &str, due_before: DateTime<Utc>, limit: u32) -> anyhow::Result<Vec<MemoryState>> {
        let rows = sqlx::query(
            "SELECT user_id, node_id, stability, difficulty, energy,
                    last_reviewed, due_at, review_count
             FROM user_memory_states
             WHERE user_id = ? AND due_at <= ?
             ORDER BY due_at ASC
             LIMIT ?"
        )
        .bind(user_id)
        .bind(due_before.timestamp_millis())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| {
            MemoryState {
                user_id: r.get("user_id"),
                node_id: r.get("node_id"),
                stability: r.get("stability"),
                difficulty: r.get("difficulty"),
                energy: r.get("energy"),
                last_reviewed: DateTime::from_timestamp_millis(r.get("last_reviewed"))
                    .unwrap_or_else(|| Utc::now()),
                due_at: DateTime::from_timestamp_millis(r.get("due_at"))
                    .unwrap_or_else(|| Utc::now()),
                review_count: r.get::<i64, _>("review_count") as u32,
            }
        }).collect())
    }

    async fn update_energy(&self, user_id: &str, node_id: &str, new_energy: f64) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE user_memory_states SET energy = ? WHERE user_id = ? AND node_id = ?"
        )
        .bind(new_energy)
        .bind(user_id)
        .bind(node_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn log_propagation(&self, event: &PropagationEvent) -> anyhow::Result<()> {
        // Insert event
        let result = sqlx::query(
            "INSERT INTO propagation_events (source_node_id, event_timestamp)
             VALUES (?, ?)"
        )
        .bind(&event.source_node_id)
        .bind(event.event_timestamp.timestamp_millis())
        .execute(&self.pool)
        .await?;

        let event_id = result.last_insert_rowid();

        // Insert details
        for detail in &event.details {
            sqlx::query(
                "INSERT INTO propagation_details (event_id, target_node_id, energy_change, reason)
                 VALUES (?, ?, ?, ?)"
            )
            .bind(event_id)
            .bind(&detail.target_node_id)
            .bind(detail.energy_change)
            .bind(&detail.reason)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn get_session_state(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT node_id FROM session_state ORDER BY session_order ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.get("node_id")).collect())
    }

    async fn save_session_state(&self, node_ids: &[String]) -> anyhow::Result<()> {
        // Clear existing
        self.clear_session_state().await?;

        // Insert new
        for (idx, node_id) in node_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO session_state (node_id, session_order) VALUES (?, ?)"
            )
            .bind(node_id)
            .bind(idx as i64)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn clear_session_state(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM session_state")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_stat(&self, key: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            "SELECT value FROM user_stats WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("value")))
    }

    async fn set_stat(&self, key: &str, value: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO user_stats (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

**File:** `rust/crates/iqrah-storage/src/user/mod.rs`

```rust
pub mod repository;

pub use repository::SqliteUserRepository;

use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

/// Initialize user database with migrations
pub async fn init_user_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
```

## Task 4.3: Update Storage lib.rs

**File:** `rust/crates/iqrah-storage/src/lib.rs`

```rust
pub mod content;
pub mod user;

pub use content::{SqliteContentRepository, init_content_db};
pub use user::{SqliteUserRepository, init_user_db};
```

## Task 4.4: Configure SQLx for Migrations

SQLx needs to know where the migrations are. Update `iqrah-storage/Cargo.toml`:

```toml
# Add this build script reference
[package]
name = "iqrah-storage"
version = "0.1.0"
edition = "2021"

# ... rest of package config

[dependencies]
# ... existing dependencies

# SQLx needs this environment variable set
# We'll use a build script to handle this
```

Create `.env` file in `iqrah-storage/`:

```bash
DATABASE_URL=sqlite::memory:
```

## Validation

### Build Storage Crate

```bash
cd /home/user/iqrah-mobile/rust
cargo build -p iqrah-storage
```

Expected: Compiles successfully.

### Check Migrations

```bash
cd /home/user/iqrah-mobile/rust/crates/iqrah-storage
sqlx database create
sqlx migrate run
```

This will create a test database and run migrations to verify they work.

## Success Criteria

- [ ] `iqrah-storage` compiles without errors
- [ ] ContentRepository trait implemented
- [ ] UserRepository trait implemented
- [ ] Both repositories use SQLx
- [ ] Migrations run successfully

## Next Step

Proceed to `05-MIGRATION-HARNESS.md`
