# Task 3.1: Add Transaction Wrapping to Review Recording

## Metadata
- **Priority:** P1 (Data Integrity)
- **Estimated Effort:** 1 day
- **Dependencies:** None
- **Agent Type:** Implementation
- **Parallelizable:** Yes (with 3.2)

## Goal

Wrap review recording and energy propagation in a single database transaction to prevent partial state on crashes or errors.

## Context

**Current Problem:**
```rust
// learning_service.rs
pub async fn record_review(&self, node_id: &str, rating: Rating) -> Result<()> {
    self.update_memory_state(node_id, rating).await?;  // Commits here ✓

    let edges = self.get_edges_from(node_id).await?;
    for edge in edges {
        self.add_energy(&edge.target_id, delta).await?;  // Each commits ✗
    }
    // If crash here, memory state updated but energy not propagated!
}
```

**Issue:** Partial state possible. User progress updated but propagation incomplete.

**Solution:** Atomic transaction wrapping all operations.

## Implementation Steps

### Step 1: Add Transaction Support to UserRepository (2 hours)

**File:** `rust/crates/iqrah-storage/src/user/repository.rs`

```rust
use sqlx::Transaction;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Sqlite>> {
        self.pool.begin().await.map_err(Into::into)
    }

    pub async fn update_memory_state_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        state: &MemoryState,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO user_memory_states (...) VALUES (...)"
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn add_energy_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        user_id: &str,
        node_id: &str,
        energy_delta: f64,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE user_memory_states SET energy = energy + ? WHERE ..."
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    pub async fn record_propagation_event_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &PropagationEvent,
    ) -> Result<()> {
        sqlx::query("INSERT INTO propagation_events (...) VALUES (...)")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
```

### Step 2: Update LearningService to Use Transactions (2 hours)

**File:** `rust/crates/iqrah-core/src/services/learning_service.rs`

```rust
pub async fn record_review(&self, user_id: &str, node_id: &str, rating: Rating) -> Result<()> {
    // Begin transaction
    let mut tx = self.user_repo.begin_transaction().await?;

    // 1. Update FSRS state
    let state = self.calculate_fsrs_state(user_id, node_id, rating).await?;
    self.user_repo.update_memory_state_tx(&mut tx, &state).await?;

    // 2. Get edges
    let edges = self.content_repo.get_edges_from(node_id).await?;

    // 3. Propagate energy
    for edge in edges {
        let energy_delta = self.sample_distribution(&edge.distribution);
        self.user_repo.add_energy_tx(&mut tx, user_id, &edge.target_id, energy_delta).await?;

        // Record propagation event
        let event = PropagationEvent {
            user_id: user_id.to_string(),
            source_node_id: node_id.to_string(),
            target_node_id: edge.target_id.clone(),
            energy_delta,
            timestamp: Utc::now(),
        };
        self.user_repo.record_propagation_event_tx(&mut tx, &event).await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(())
}
```

### Step 3: Add Rollback Tests (1 hour)

**File:** `rust/crates/iqrah-core/tests/transaction_test.rs`

```rust
#[tokio::test]
async fn test_review_rollback_on_error() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let learning_service = LearningService::new(content_repo, user_repo.clone());

    // Simulate error during propagation (e.g., invalid target node)
    let result = learning_service
        .record_review("default", "1:1:memorization", Rating::Good)
        .await;

    // If any step fails, entire transaction should rollback
    if result.is_err() {
        // Verify memory state was NOT updated
        let state = user_repo
            .get_memory_state("default", "1:1:memorization")
            .await
            .unwrap();

        assert!(state.is_none() || state.unwrap().review_count == 0);
    }
}
```

## Verification Plan

- [ ] Transaction methods added to UserRepository
- [ ] LearningService uses transactions
- [ ] All operations wrapped in single transaction
- [ ] Test: Successful review commits all changes
- [ ] Test: Failed review rolls back all changes
- [ ] Integration test with simulated crash
- [ ] Performance: No significant slowdown

## Success Criteria

- [ ] All review operations atomic
- [ ] Rollback tests pass
- [ ] No partial state possible
- [ ] All existing tests pass
- [ ] CI checks pass

## Related Files

**Modify:**
- `/rust/crates/iqrah-storage/src/user/repository.rs`
- `/rust/crates/iqrah-core/src/services/learning_service.rs`

**Create:**
- `/rust/crates/iqrah-core/tests/transaction_test.rs`
