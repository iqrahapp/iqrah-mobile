# Task 3.3: Add Graph Update Mechanism

## Metadata
- **Priority:** P1 (Data Integrity)
- **Estimated Effort:** 1 day
- **Dependencies:** Task 3.2 (Referential integrity validation)
- **Agent Type:** Implementation
- **Parallelizable:** No (depends on 3.2)

## Goal

Implement a safe mechanism to update the knowledge graph (monthly updates) via erase/replace pattern while preserving user progress and validating node ID stability.

## Context

**Monthly Update Scenario:**
1. R&D team generates new graph with:
   - Improved PageRank scores
   - New edges
   - Same node IDs (stability guaranteed by Task 1.5)
2. Need to update content.db without breaking user progress

**Update Strategy: Erase & Replace**
```sql
BEGIN TRANSACTION;
DELETE FROM edges;
DELETE FROM node_metadata;
DELETE FROM goals;
DELETE FROM node_goals;
-- INSERT new data
COMMIT;
```

**Safety Requirements:**
- Validate node IDs before deletion
- Ensure user progress compatible
- Rollback on errors
- Report changes

## Implementation Steps

### Step 1: Create Graph Update Function (2 hours)

**File:** `rust/crates/iqrah-storage/src/content/graph_update.rs` (NEW)

```rust
use sqlx::{SqlitePool, Transaction};
use std::path::Path;

pub struct GraphUpdate {
    pool: SqlitePool,
}

pub struct UpdateStats {
    pub nodes_before: usize,
    pub nodes_after: usize,
    pub edges_before: usize,
    pub edges_after: usize,
    pub new_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
}

impl GraphUpdate {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn update_from_sql_file(&self, sql_path: &Path) -> Result<UpdateStats> {
        // 1. Collect current graph stats
        let stats_before = self.collect_graph_stats().await?;

        // 2. Read new graph SQL
        let new_graph_sql = std::fs::read_to_string(sql_path)?;

        // 3. Parse and validate (extract node IDs from SQL)
        let new_node_ids = self.extract_node_ids_from_sql(&new_graph_sql)?;

        // 4. Check for removed nodes
        let removed_nodes = stats_before.node_ids
            .iter()
            .filter(|id| !new_node_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();

        if !removed_nodes.is_empty() {
            return Err(GraphUpdateError::RemovedNodes {
                count: removed_nodes.len(),
                sample: removed_nodes.iter().take(5).cloned().collect(),
            });
        }

        // 5. Begin transaction
        let mut tx = self.pool.begin().await?;

        // 6. Delete old graph
        self.delete_graph_tables(&mut tx).await?;

        // 7. Insert new graph
        sqlx::raw_sql(&new_graph_sql)
            .execute(&mut *tx)
            .await?;

        // 8. Collect new stats
        let stats_after = self.collect_graph_stats_tx(&mut tx).await?;

        // 9. Commit
        tx.commit().await?;

        // 10. Return stats
        Ok(UpdateStats {
            nodes_before: stats_before.node_ids.len(),
            nodes_after: stats_after.node_ids.len(),
            edges_before: stats_before.edge_count,
            edges_after: stats_after.edge_count,
            new_nodes: new_node_ids.iter()
                .filter(|id| !stats_before.node_ids.contains(*id))
                .cloned()
                .collect(),
            removed_nodes,
        })
    }

    async fn delete_graph_tables(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        sqlx::query("DELETE FROM edges").execute(&mut **tx).await?;
        sqlx::query("DELETE FROM node_metadata").execute(&mut **tx).await?;
        sqlx::query("DELETE FROM goals").execute(&mut **tx).await?;
        sqlx::query("DELETE FROM node_goals").execute(&mut **tx).await?;
        Ok(())
    }

    async fn collect_graph_stats(&self) -> Result<GraphStats> {
        let node_ids: Vec<String> = sqlx::query_scalar("SELECT DISTINCT node_id FROM node_metadata")
            .fetch_all(&self.pool)
            .await?;

        let edge_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges")
            .fetch_one(&self.pool)
            .await?;

        Ok(GraphStats {
            node_ids,
            edge_count: edge_count as usize,
        })
    }

    fn extract_node_ids_from_sql(&self, sql: &str) -> Result<Vec<String>> {
        // Parse INSERT statements to extract node IDs
        // Regex: INSERT INTO node_metadata \(node_id, ...\) VALUES \('([^']+)'
        let re = regex::Regex::new(r"'([^']+)'").unwrap();

        let mut node_ids = std::collections::HashSet::new();
        for line in sql.lines() {
            if line.contains("INSERT INTO node_metadata") {
                if let Some(caps) = re.captures(line) {
                    node_ids.insert(caps[1].to_string());
                }
            }
        }

        Ok(node_ids.into_iter().collect())
    }
}
```

### Step 2: Add CLI Command (1 hour)

**File:** `rust/crates/iqrah-cli/src/commands/update_graph.rs`

```rust
pub async fn update_graph(sql_path: PathBuf, content_db: PathBuf) -> Result<()> {
    println!("Updating knowledge graph...");
    println!("  Source: {}", sql_path.display());
    println!("  Target: {}", content_db.display());

    let pool = SqlitePool::connect(content_db.to_str().unwrap()).await?;
    let updater = GraphUpdate::new(pool);

    let stats = updater.update_from_sql_file(&sql_path).await?;

    println!("\n✅ Graph updated successfully!");
    println!("\nStatistics:");
    println!("  Nodes before: {}", stats.nodes_before);
    println!("  Nodes after:  {}", stats.nodes_after);
    println!("  New nodes:    {}", stats.new_nodes.len());
    println!("  Edges before: {}", stats.edges_before);
    println!("  Edges after:  {}", stats.edges_after);

    if !stats.new_nodes.is_empty() {
        println!("\nNew nodes (sample):");
        for node in stats.new_nodes.iter().take(5) {
            println!("  + {}", node);
        }
        if stats.new_nodes.len() > 5 {
            println!("  ... and {} more", stats.new_nodes.len() - 5);
        }
    }

    Ok(())
}
```

**Usage:**
```bash
cargo run --bin iqrah-cli -- update-graph --file new_graph.sql
```

### Step 3: Add Tests (2 hours)

**File:** `rust/crates/iqrah-storage/tests/graph_update_test.rs`

```rust
#[tokio::test]
async fn test_graph_update_success() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("content.db");

    // Initialize with old graph
    let pool = init_content_db(db_path.to_str().unwrap()).await.unwrap();

    // Create new graph SQL
    let new_graph_sql = r#"
        INSERT INTO node_metadata (node_id, key, value) VALUES ('1:1', 'score', 0.9);
        INSERT INTO edges (source_id, target_id, edge_type, ...) VALUES ('1:1', '1:2', 0, ...);
    "#;

    let sql_file = tmp.path().join("new_graph.sql");
    std::fs::write(&sql_file, new_graph_sql).unwrap();

    // Update graph
    let updater = GraphUpdate::new(pool);
    let stats = updater.update_from_sql_file(&sql_file).await.unwrap();

    assert!(stats.nodes_after > 0);
    assert!(stats.edges_after > 0);
}

#[tokio::test]
async fn test_graph_update_rejects_removed_nodes() {
    // Setup with existing user progress
    let (content_pool, user_pool) = setup_test_dbs().await;

    // User has progress on node "1:1:memorization"
    create_user_progress(&user_pool, "1:1:memorization").await;

    // New graph removes "1:1:memorization"
    let new_graph_sql = "/* SQL without 1:1:memorization */";
    let sql_file = write_temp_sql(new_graph_sql);

    let updater = GraphUpdate::new(content_pool);
    let result = updater.update_from_sql_file(&sql_file).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GraphUpdateError::RemovedNodes { .. }));
}

#[tokio::test]
async fn test_graph_update_rollback_on_error() {
    let pool = setup_test_db().await;

    let stats_before = collect_stats(&pool).await;

    // Invalid SQL should cause rollback
    let invalid_sql = "INSERT INTO node_metadata VALUES (INVALID);";
    let sql_file = write_temp_sql(invalid_sql);

    let updater = GraphUpdate::new(pool.clone());
    let result = updater.update_from_sql_file(&sql_file).await;

    assert!(result.is_err());

    // Graph should be unchanged
    let stats_after = collect_stats(&pool).await;
    assert_eq!(stats_before.node_count, stats_after.node_count);
}
```

## Verification Plan

- [ ] Graph update function implemented
- [ ] CLI command works
- [ ] Test: Successful update
- [ ] Test: Rejected when nodes removed
- [ ] Test: Rollback on error
- [ ] User progress preserved after update
- [ ] Performance: Update completes < 5 seconds for 10MB SQL

## Success Criteria

- [ ] Graph update mechanism implemented
- [ ] Node stability validation (rejects removed nodes)
- [ ] Transaction rollback on errors
- [ ] CLI command with clear output
- [ ] Tests pass (3+ cases)
- [ ] User progress preserved
- [ ] CI checks pass

## Related Files

**Create:**
- `/rust/crates/iqrah-storage/src/content/graph_update.rs`
- `/rust/crates/iqrah-cli/src/commands/update_graph.rs`
- `/rust/crates/iqrah-storage/tests/graph_update_test.rs`

**Modify:**
- `/rust/crates/iqrah-storage/src/error.rs` (add GraphUpdateError)
