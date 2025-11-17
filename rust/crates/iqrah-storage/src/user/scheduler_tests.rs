/// Unit tests for scheduler v2 user repository methods
///
/// Tests cover:
/// - get_parent_energies: Query correctness and chunking (0, 1, 500, 501, 1000+ nodes)
/// - get_memory_basics: Query correctness, chunking, and next_due_ts handling
/// - get_bandit_arms: Basic retrieval and profile name parsing
/// - update_bandit_arm: Upsert behavior and timestamp updates
use super::repository::SqliteUserRepository;
use iqrah_core::{scheduler_v2::ProfileName, UserRepository};
use sqlx::{query, SqlitePool};

/// Create an in-memory test database with user schema
async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // Create tables
    query(
        "CREATE TABLE user_memory_states (
            user_id TEXT NOT NULL,
            content_key TEXT NOT NULL,
            stability REAL NOT NULL,
            difficulty REAL NOT NULL,
            energy REAL NOT NULL,
            last_reviewed INTEGER NOT NULL,
            due_at INTEGER NOT NULL,
            review_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (user_id, content_key)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create user_memory_states table");

    query(
        "CREATE TABLE user_bandit_state (
            user_id TEXT NOT NULL,
            goal_group TEXT NOT NULL,
            profile_name TEXT NOT NULL,
            successes REAL NOT NULL DEFAULT 1.0,
            failures REAL NOT NULL DEFAULT 1.0,
            last_updated INTEGER NOT NULL,
            PRIMARY KEY (user_id, goal_group, profile_name)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create user_bandit_state table");

    pool
}

// ============================================================================
// get_parent_energies Tests
// ============================================================================

#[tokio::test]
async fn test_get_parent_energies_empty_input() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool);

    let result = repo
        .get_parent_energies("user1", &[])
        .await
        .expect("Should succeed with empty input");

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_get_parent_energies_single_node() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert memory state
    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES ('user1', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700000000000, 10)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = repo
        .get_parent_energies("user1", &[String::from("1:1")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get("1:1").unwrap(), &0.8);
}

#[tokio::test]
async fn test_get_parent_energies_missing_node_not_in_result() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert one node
    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES ('user1', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700000000000, 10)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query for two nodes (one missing)
    let result = repo
        .get_parent_energies("user1", &[String::from("1:1"), String::from("1:2")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1); // Only found node in result
    assert_eq!(result.get("1:1").unwrap(), &0.8);
    assert!(!result.contains_key("1:2"));
}

#[tokio::test]
async fn test_get_parent_energies_different_users() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert for two users
    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES ('user1', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700000000000, 10)",
    )
    .execute(&pool)
    .await
    .unwrap();

    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES ('user2', '1:1', 3.0, 4.0, 0.5, 1700000000000, 1700000000000, 5)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Query for user1
    let result = repo
        .get_parent_energies("user1", &[String::from("1:1")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get("1:1").unwrap(), &0.8); // user1's energy, not user2's
}

#[tokio::test]
async fn test_get_parent_energies_chunking_500_nodes() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert 500 nodes
    let mut node_ids = Vec::new();
    for i in 1..=500 {
        let node_id = format!("node_{}", i);
        node_ids.push(node_id.clone());
        let energy = (i as f32) / 1000.0; // 0.001 to 0.500

        query(
            "INSERT INTO user_memory_states
            (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
            VALUES ('user1', ?, 5.0, 3.0, ?, 1700000000000, 1700000000000, 10)",
        )
        .bind(&node_id)
        .bind(energy)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = repo
        .get_parent_energies("user1", &node_ids)
        .await
        .expect("Should succeed with 500 nodes");

    assert_eq!(result.len(), 500);
    for i in 1..=500 {
        let node_id = format!("node_{}", i);
        let expected_energy = (i as f32) / 1000.0;
        assert!((result.get(&node_id).unwrap() - expected_energy).abs() < 0.0001);
    }
}

#[tokio::test]
async fn test_get_parent_energies_chunking_501_nodes() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert 501 nodes (tests chunking across 2 batches)
    let mut node_ids = Vec::new();
    for i in 1..=501 {
        let node_id = format!("node_{}", i);
        node_ids.push(node_id.clone());
        let energy = (i as f32) / 1000.0;

        query(
            "INSERT INTO user_memory_states
            (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
            VALUES ('user1', ?, 5.0, 3.0, ?, 1700000000000, 1700000000000, 10)",
        )
        .bind(&node_id)
        .bind(energy)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = repo
        .get_parent_energies("user1", &node_ids)
        .await
        .expect("Should succeed with 501 nodes (2 chunks)");

    assert_eq!(result.len(), 501);
    for i in 1..=501 {
        let node_id = format!("node_{}", i);
        let expected_energy = (i as f32) / 1000.0;
        assert!((result.get(&node_id).unwrap() - expected_energy).abs() < 0.0001);
    }
}

#[tokio::test]
async fn test_get_parent_energies_chunking_1000_nodes() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert 1000 nodes (tests chunking across 2 batches: 500 + 500)
    let mut node_ids = Vec::new();
    for i in 1..=1000 {
        let node_id = format!("node_{}", i);
        node_ids.push(node_id.clone());
        let energy = (i as f32) / 1000.0;

        query(
            "INSERT INTO user_memory_states
            (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
            VALUES ('user1', ?, 5.0, 3.0, ?, 1700000000000, 1700000000000, 10)",
        )
        .bind(&node_id)
        .bind(energy)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = repo
        .get_parent_energies("user1", &node_ids)
        .await
        .expect("Should succeed with 1000 nodes (2 chunks)");

    assert_eq!(result.len(), 1000);
    for i in 1..=1000 {
        let node_id = format!("node_{}", i);
        let expected_energy = (i as f32) / 1000.0;
        assert!((result.get(&node_id).unwrap() - expected_energy).abs() < 0.0001);
    }
}

// ============================================================================
// get_memory_basics Tests
// ============================================================================

#[tokio::test]
async fn test_get_memory_basics_empty_input() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool);

    let result = repo
        .get_memory_basics("user1", &[])
        .await
        .expect("Should succeed with empty input");

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_get_memory_basics_single_node() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert memory state
    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES ('user1', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700500000000, 10)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = repo
        .get_memory_basics("user1", &[String::from("1:1")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    let basics = result.get("1:1").unwrap();
    assert_eq!(basics.energy, 0.8);
    assert_eq!(basics.next_due_ts, 1700500000000);
}

#[tokio::test]
async fn test_get_memory_basics_multiple_nodes() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert multiple memory states
    query(
        "INSERT INTO user_memory_states
        (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
        VALUES
        ('user1', '1:1', 5.0, 3.0, 0.8, 1700000000000, 1700500000000, 10),
        ('user1', '1:2', 4.0, 3.5, 0.6, 1700100000000, 1700600000000, 8),
        ('user1', '1:3', 2.0, 4.0, 0.25, 1700200000000, 1700700000000, 3)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = repo
        .get_memory_basics(
            "user1",
            &[
                String::from("1:1"),
                String::from("1:2"),
                String::from("1:3"),
            ],
        )
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 3);

    let basics1 = result.get("1:1").unwrap();
    assert_eq!(basics1.energy, 0.8);
    assert_eq!(basics1.next_due_ts, 1700500000000);

    let basics2 = result.get("1:2").unwrap();
    assert_eq!(basics2.energy, 0.6);
    assert_eq!(basics2.next_due_ts, 1700600000000);

    let basics3 = result.get("1:3").unwrap();
    assert_eq!(basics3.energy, 0.25);
    assert_eq!(basics3.next_due_ts, 1700700000000);
}

#[tokio::test]
async fn test_get_memory_basics_chunking_501_nodes() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert 501 nodes with different due timestamps
    let mut node_ids = Vec::new();
    for i in 1..=501 {
        let node_id = format!("node_{}", i);
        node_ids.push(node_id.clone());
        let energy = (i as f32) / 1000.0;
        let due_at = 1700000000000 + (i as i64) * 1000; // Different timestamps

        query(
            "INSERT INTO user_memory_states
            (user_id, content_key, stability, difficulty, energy, last_reviewed, due_at, review_count)
            VALUES ('user1', ?, 5.0, 3.0, ?, 1700000000000, ?, 10)",
        )
        .bind(&node_id)
        .bind(energy)
        .bind(due_at)
        .execute(&pool)
        .await
        .unwrap();
    }

    let result = repo
        .get_memory_basics("user1", &node_ids)
        .await
        .expect("Should succeed with 501 nodes (2 chunks)");

    assert_eq!(result.len(), 501);
    for i in 1..=501 {
        let node_id = format!("node_{}", i);
        let expected_energy = (i as f32) / 1000.0;
        let expected_due = 1700000000000 + (i as i64) * 1000;

        let basics = result.get(&node_id).unwrap();
        assert!((basics.energy - expected_energy).abs() < 0.0001);
        assert_eq!(basics.next_due_ts, expected_due);
    }
}

// ============================================================================
// get_bandit_arms Tests
// ============================================================================

#[tokio::test]
async fn test_get_bandit_arms_empty_result() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool);

    let arms = repo
        .get_bandit_arms("user1", "memorization")
        .await
        .expect("Should succeed with empty result");

    assert_eq!(arms.len(), 0);
}

#[tokio::test]
async fn test_get_bandit_arms_single_arm() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert bandit arm
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1700000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let arms = repo
        .get_bandit_arms("user1", "memorization")
        .await
        .expect("Should succeed");

    assert_eq!(arms.len(), 1);
    let arm = &arms[0];
    assert_eq!(arm.profile_name, ProfileName::Balanced);
    assert_eq!(arm.successes, 5.0);
    assert_eq!(arm.failures, 3.0);
}

#[tokio::test]
async fn test_get_bandit_arms_multiple_profiles() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert multiple profiles
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES
        ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1700000000000),
        ('user1', 'memorization', 'FoundationHeavy', 8.0, 2.0, 1700000000000),
        ('user1', 'memorization', 'UrgencyHeavy', 3.0, 7.0, 1700000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let arms = repo
        .get_bandit_arms("user1", "memorization")
        .await
        .expect("Should succeed");

    assert_eq!(arms.len(), 3);

    // Check all profiles are present
    let profile_names: Vec<ProfileName> = arms.iter().map(|a| a.profile_name).collect();
    assert!(profile_names.contains(&ProfileName::Balanced));
    assert!(profile_names.contains(&ProfileName::FoundationHeavy));
    assert!(profile_names.contains(&ProfileName::UrgencyHeavy));
}

#[tokio::test]
async fn test_get_bandit_arms_filters_by_user_and_goal_group() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert for different users and goal groups
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES
        ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1700000000000),
        ('user2', 'memorization', 'Balanced', 8.0, 2.0, 1700000000000),
        ('user1', 'understanding', 'Balanced', 3.0, 7.0, 1700000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let arms = repo
        .get_bandit_arms("user1", "memorization")
        .await
        .expect("Should succeed");

    assert_eq!(arms.len(), 1); // Only user1 + memorization
    assert_eq!(arms[0].successes, 5.0);
}

#[tokio::test]
async fn test_get_bandit_arms_ignores_invalid_profile_names() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert valid and invalid profile names
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES
        ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1700000000000),
        ('user1', 'memorization', 'invalid_profile', 8.0, 2.0, 1700000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let arms = repo
        .get_bandit_arms("user1", "memorization")
        .await
        .expect("Should succeed");

    assert_eq!(arms.len(), 1); // Only valid profile
    assert_eq!(arms[0].profile_name, ProfileName::Balanced);
}

// ============================================================================
// update_bandit_arm Tests
// ============================================================================

#[tokio::test]
async fn test_update_bandit_arm_insert_new() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    repo.update_bandit_arm("user1", "memorization", "Balanced", 5.0, 3.0)
        .await
        .expect("Should succeed");

    // Verify insert
    let arms = repo.get_bandit_arms("user1", "memorization").await.unwrap();

    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0].profile_name, ProfileName::Balanced);
    assert_eq!(arms[0].successes, 5.0);
    assert_eq!(arms[0].failures, 3.0);
}

#[tokio::test]
async fn test_update_bandit_arm_update_existing() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert initial state
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1700000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Update
    repo.update_bandit_arm("user1", "memorization", "Balanced", 10.0, 5.0)
        .await
        .expect("Should succeed");

    // Verify update
    let arms = repo.get_bandit_arms("user1", "memorization").await.unwrap();

    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0].successes, 10.0); // Updated
    assert_eq!(arms[0].failures, 5.0); // Updated
}

#[tokio::test]
async fn test_update_bandit_arm_updates_timestamp() {
    let pool = create_test_db().await;
    let repo = SqliteUserRepository::new(pool.clone());

    // Insert initial state with old timestamp
    query(
        "INSERT INTO user_bandit_state
        (user_id, goal_group, profile_name, successes, failures, last_updated)
        VALUES ('user1', 'memorization', 'Balanced', 5.0, 3.0, 1000000000000)",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Update (should update timestamp)
    repo.update_bandit_arm("user1", "memorization", "Balanced", 10.0, 5.0)
        .await
        .expect("Should succeed");

    // Verify timestamp was updated (should be much later than 1000000000000)
    let result: (i64,) = sqlx::query_as(
        "SELECT last_updated FROM user_bandit_state
         WHERE user_id = 'user1' AND goal_group = 'memorization' AND profile_name = 'Balanced'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(result.0 > 1000000000000); // Timestamp should be updated
}
