# Task 3.2: Add Referential Integrity Validation

## Metadata
- **Priority:** P1 (Data Integrity)
- **Estimated Effort:** 1 day
- **Dependencies:** Task 1.4 (Node ID parsing)
- **Agent Type:** Implementation
- **Parallelizable:** Yes (with 3.1)

## Goal

Add application-level validation to ensure node_id references in user.db always point to valid nodes in content.db, preventing orphaned user progress.

## Context

**The Problem:**
Two separate databases means no foreign key constraints:
```sql
-- user.db
user_memory_states(user_id, node_id, ...)  -- node_id is just a string

-- content.db (separate database)
-- No FK constraint possible between databases
```

**Risk:** User progress saved for non-existent node_id → orphaned data.

**Solution:** Application-level validation before saving.

## Implementation Steps

### Step 1: Add Validation to UserRepository (2 hours)

**File:** `rust/crates/iqrah-storage/src/user/repository.rs`

```rust
impl SqliteUserRepository {
    /// Validate that node_id exists before saving user progress
    async fn validate_node_exists(
        &self,
        content_repo: &dyn ContentRepository,
        node_id: &str,
    ) -> Result<()> {
        let node = content_repo.get_node(node_id).await?;

        if node.is_none() {
            return Err(StorageError::InvalidNodeReference {
                node_id: node_id.to_string(),
                reason: "Node does not exist in content database".to_string(),
            });
        }

        Ok(())
    }
}
```

### Step 2: Update LearningService with Validation (1 hour)

**File:** `rust/crates/iqrah-core/src/services/learning_service.rs`

```rust
pub async fn record_review(&self, user_id: &str, node_id: &str, rating: Rating) -> Result<()> {
    // Validate node exists
    if self.content_repo.get_node(node_id).await?.is_none() {
        return Err(LearningError::InvalidNode {
            node_id: node_id.to_string(),
        });
    }

    // Proceed with transaction...
}
```

### Step 3: Add Orphan Detection CLI Command (2 hours)

**File:** `rust/crates/iqrah-cli/src/commands/check_integrity.rs`

```rust
pub async fn check_integrity(content_repo: Arc<dyn ContentRepository>, user_repo: Arc<dyn UserRepository>) -> Result<()> {
    println!("Checking database integrity...\n");

    // Get all unique node_ids from user progress
    let node_ids = user_repo.get_all_node_ids("default").await?;

    let mut orphaned = vec![];

    for node_id in &node_ids {
        if content_repo.get_node(node_id).await?.is_none() {
            orphaned.push(node_id.clone());
        }
    }

    if orphaned.is_empty() {
        println!("✅ No orphaned records found.");
    } else {
        println!("⚠️  Found {} orphaned records:", orphaned.len());
        for node_id in &orphaned[..orphaned.len().min(10)] {
            println!("  - {}", node_id);
        }
        if orphaned.len() > 10 {
            println!("  ... and {} more", orphaned.len() - 10);
        }
    }

    Ok(())
}
```

**Register command:**
```bash
cargo run --bin iqrah-cli -- check-integrity
```

### Step 4: Add Tests (1 hour)

**File:** `rust/crates/iqrah-core/tests/referential_integrity_test.rs`

```rust
#[tokio::test]
async fn test_reject_invalid_node_id() {
    let (content_repo, user_repo) = setup_test_repos().await;
    let learning_service = LearningService::new(content_repo, user_repo);

    let result = learning_service
        .record_review("default", "INVALID:NODE:ID", Rating::Good)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_orphan_detection() {
    let (content_repo, user_repo) = setup_test_repos().await;

    // Manually insert orphaned record (bypassing validation)
    user_repo.force_insert_state("default", "999:999").await.unwrap();

    // Check integrity should detect it
    let orphans = check_for_orphans(&content_repo, &user_repo).await.unwrap();

    assert!(orphans.contains(&"999:999".to_string()));
}
```

## Verification Plan

- [ ] Validation added to save operations
- [ ] Invalid node_id rejected with clear error
- [ ] `check-integrity` CLI command works
- [ ] Orphan detection finds test cases
- [ ] Tests pass
- [ ] Performance acceptable (<10ms validation per save)

## Success Criteria

- [ ] Application-level FK enforcement
- [ ] CLI orphan detection tool
- [ ] Tests pass (3+ cases)
- [ ] Error messages clear
- [ ] CI checks pass

## Related Files

**Modify:**
- `/rust/crates/iqrah-storage/src/user/repository.rs`
- `/rust/crates/iqrah-core/src/services/learning_service.rs`
- `/rust/crates/iqrah-storage/src/error.rs`

**Create:**
- `/rust/crates/iqrah-cli/src/commands/check_integrity.rs`
- `/rust/crates/iqrah-core/tests/referential_integrity_test.rs`
