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

### Content Update Strategy

The core of the graph update mechanism is the two-database architecture, which is designed for safe, seamless content updates.

#### 1. `content.db` Replacement

The `content.db` file, which contains the entire knowledge graph, is treated as an **immutable artifact**. The update process does not involve `UPDATE` or `DELETE` statements. Instead, the entire file is replaced.

- **Process**:
    1. The Python generator creates a new `content.db` file from scratch.
    2. This new database file is shipped to the user's device.
    3. The application replaces the old `content.db` with the new one.
- **Impact on IDs**:
    - All internal **integer node IDs will change** during this process, as they are auto-incrementing primary keys.
    - The **string unique keys (`ukeys`) remain stable**, as they are guaranteed by the generator's validation logic.

#### 2. `user.db` Stability

The `user.db` is **never replaced**. It stores all user-specific data, such as memory states, and persists across all content updates.

- **Key Design**: User data is linked to nodes via the stable **string `ukey`**, not the volatile integer ID.
    ```sql
    CREATE TABLE user_memory_states (
        user_id TEXT NOT NULL,
        node_ukey TEXT NOT NULL,  -- Stable string key, immune to content updates
        stability REAL NOT NULL,
        -- ... other state fields ...
        PRIMARY KEY (user_id, node_ukey)
    ) STRICT, WITHOUT ROWID;
    ```

### Application Logic at Startup

When the application loads after a content update, it must re-link the user's progress from `user.db` to the new graph in `content.db`.

1. **Load User State**: Query `user.db` to get the list of `node_ukey`s for which the user has progress.
2. **Resolve New IDs**: For each `node_ukey`, use the `NodeRegistry` (which is connected to the new `content.db`) to look up the new integer `id`.
3. **Perform Operations**: All subsequent graph operations for the session (fetching nodes, propagating energy, etc.) will use the newly resolved integer IDs.

This ensures that user progress is seamlessly carried over, even though the internal graph structure and primary keys have completely changed.

### CLI Command for Verification

A CLI command should be implemented to simulate and verify this update process.

**File:** `rust/crates/iqrah-cli/src/commands/verify_update.rs`

```rust
pub async fn verify_update(old_db: PathBuf, new_db: PathBuf, user_db: PathBuf) -> Result<()> {
    println!("Verifying graph update compatibility...");

    // Setup repositories for both old and new content versions
    let old_content_repo = setup_content_repo(&old_db).await?;
    let new_content_repo = setup_content_repo(&new_db).await?;
    let user_repo = setup_user_repo(&user_db).await?;

    // 1. Get user progress (list of ukeys) from user.db
    let user_nodes = user_repo.get_all_user_nodes("default_user").await?;

    // 2. For each ukey, check if it exists in the OLD and NEW content.db
    for ukey in user_nodes {
        let old_node = old_content_repo.get_node(&ukey).await?;
        let new_node = new_content_repo.get_node(&ukey).await?;

        if old_node.is_none() {
            // This should not happen if data is consistent
            println!("⚠️ Warning: User has progress for a node that doesn't exist in the OLD DB: {}", ukey);
        }
        if new_node.is_none() {
            // CRITICAL ERROR: A node the user had progress on was removed in the update
            return Err(anyhow!("Error: Node '{}' was removed in the new version, leading to data loss.", ukey));
        }
    }

    println!("✅ Verification successful: All user progress can be migrated.");
    Ok(())
}
```

**Usage:**
```bash
cargo run --bin iqrah-cli -- verify-update \
    --old-db /path/to/old_content.db \
    --new-db /path/to/new_content.db \
    --user-db /path/to/user.db
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
        INSERT INTO node_metadata (node_id, key, value) VALUES ('VERSE:1:1', 'score', 0.9);
        INSERT INTO edges (source_id, target_id, edge_type, ...) VALUES ('VERSE:1:1', 'VERSE:1:2', 0, ...);
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

    // User has progress on node "VERSE:1:1:memorization"
    create_user_progress(&user_pool, "VERSE:1:1:memorization").await;

    // New graph removes "VERSE:1:1:memorization"
    let new_graph_sql = "/* SQL without VERSE:1:1:memorization */";
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
