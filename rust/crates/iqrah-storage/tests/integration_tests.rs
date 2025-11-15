use iqrah_storage::{init_content_db, init_user_db, SqliteContentRepository, SqliteUserRepository};
use iqrah_core::{ContentRepository, UserRepository, MemoryState, NodeType};
use chrono::Utc;
use sqlx::Row;

#[tokio::test]
async fn test_content_db_initialization() {
    let pool = init_content_db(":memory:").await.unwrap();
    let repo = SqliteContentRepository::new(pool);

    // Verify schema was created
    let node = repo.get_node("test").await.unwrap();
    assert!(node.is_none(), "Should return None for non-existent node");
}

#[tokio::test]
async fn test_user_db_initialization_and_migrations() {
    let pool = init_user_db(":memory:").await.unwrap();

    // Check that migrations ran successfully
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'schema_version'")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(row.is_some(), "Migration v2 should have created app_settings table");

    let version: String = row.unwrap().get("value");
    assert_eq!(version, "2", "Schema version should be 2 after migrations");
}

#[tokio::test]
async fn test_content_repository_crud() {
    let pool = init_content_db(":memory:").await.unwrap();

    // Insert test data manually
    sqlx::query("INSERT INTO nodes VALUES ('node1', 'word_instance', 0)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'arabic', 'بِسْمِ')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO node_metadata VALUES ('node1', 'translation', 'In the name')")
        .execute(&pool)
        .await
        .unwrap();

    // Test repository
    let repo = SqliteContentRepository::new(pool);

    // Test get_node
    let node = repo.get_node("node1").await.unwrap();
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.id, "node1");
    assert_eq!(node.node_type, NodeType::WordInstance);

    // Test get_metadata
    let arabic = repo.get_metadata("node1", "arabic").await.unwrap();
    assert_eq!(arabic, Some("بِسْمِ".to_string()));

    // Test get_all_metadata
    let all = repo.get_all_metadata("node1").await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get("arabic").unwrap(), "بِسْمِ");
    assert_eq!(all.get("translation").unwrap(), "In the name");

    // Test node_exists
    assert!(repo.node_exists("node1").await.unwrap());
    assert!(!repo.node_exists("nonexistent").await.unwrap());
}

#[tokio::test]
async fn test_user_repository_memory_states() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Create a memory state
    let state = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node1".to_string(),
        stability: 1.5,
        difficulty: 5.0,
        energy: 0.7,
        last_reviewed: Utc::now(),
        due_at: Utc::now(),
        review_count: 3,
    };

    // Save it
    repo.save_memory_state(&state).await.unwrap();

    // Retrieve it
    let retrieved = repo.get_memory_state("user1", "node1").await.unwrap();
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.user_id, "user1");
    assert_eq!(retrieved.node_id, "node1");
    assert_eq!(retrieved.stability, 1.5);
    assert_eq!(retrieved.difficulty, 5.0);
    assert_eq!(retrieved.energy, 0.7);
    assert_eq!(retrieved.review_count, 3);

    // Update it
    let mut updated = state.clone();
    updated.energy = 0.9;
    updated.review_count = 4;

    repo.save_memory_state(&updated).await.unwrap();

    let retrieved = repo.get_memory_state("user1", "node1").await.unwrap().unwrap();
    assert_eq!(retrieved.energy, 0.9);
    assert_eq!(retrieved.review_count, 4);
}

#[tokio::test]
async fn test_user_repository_get_due_states() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let now = Utc::now();

    // Create overdue state
    let overdue = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node1".to_string(),
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now - chrono::Duration::hours(1), // Overdue
        review_count: 1,
    };

    // Create future state
    let future = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node2".to_string(),
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: now,
        due_at: now + chrono::Duration::hours(1), // Not due yet
        review_count: 1,
    };

    repo.save_memory_state(&overdue).await.unwrap();
    repo.save_memory_state(&future).await.unwrap();

    // Get due states
    let due = repo.get_due_states("user1", now, 10).await.unwrap();

    assert_eq!(due.len(), 1, "Should only return overdue items");
    assert_eq!(due[0].node_id, "node1");
}

#[tokio::test]
async fn test_user_repository_stats() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Set stat
    repo.set_stat("reviews_today", "42").await.unwrap();

    // Get stat
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("42".to_string()));

    // Update stat
    repo.set_stat("reviews_today", "43").await.unwrap();
    let value = repo.get_stat("reviews_today").await.unwrap();
    assert_eq!(value, Some("43".to_string()));

    // Non-existent stat
    let value = repo.get_stat("nonexistent").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_user_repository_session_state() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    let nodes = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];

    // Save session
    repo.save_session_state(&nodes).await.unwrap();

    // Retrieve session
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved[0], "node1");
    assert_eq!(retrieved[1], "node2");
    assert_eq!(retrieved[2], "node3");

    // Clear session
    repo.clear_session_state().await.unwrap();
    let retrieved = repo.get_session_state().await.unwrap();
    assert_eq!(retrieved.len(), 0);
}

#[tokio::test]
async fn test_update_energy() {
    let pool = init_user_db(":memory:").await.unwrap();
    let repo = SqliteUserRepository::new(pool);

    // Create initial state
    let state = MemoryState {
        user_id: "user1".to_string(),
        node_id: "node1".to_string(),
        stability: 1.0,
        difficulty: 5.0,
        energy: 0.5,
        last_reviewed: Utc::now(),
        due_at: Utc::now(),
        review_count: 1,
    };

    repo.save_memory_state(&state).await.unwrap();

    // Update just the energy
    repo.update_energy("user1", "node1", 0.8).await.unwrap();

    // Verify energy was updated
    let updated = repo.get_memory_state("user1", "node1").await.unwrap().unwrap();
    assert_eq!(updated.energy, 0.8);
    assert_eq!(updated.stability, 1.0); // Other fields unchanged
}

#[tokio::test]
async fn test_two_database_integration() {
    // This test demonstrates the two-database architecture working together

    // Initialize both databases
    let content_pool = init_content_db(":memory:").await.unwrap();
    let user_pool = init_user_db(":memory:").await.unwrap();

    // Insert test content
    sqlx::query("INSERT INTO nodes VALUES ('word1', 'word_instance', 0)")
        .execute(&content_pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO node_metadata VALUES ('word1', 'arabic', 'كتاب')")
        .execute(&content_pool)
        .await
        .unwrap();

    // Create repositories
    let content_repo = SqliteContentRepository::new(content_pool);
    let user_repo = SqliteUserRepository::new(user_pool);

    // Verify content.db has the node
    let node = content_repo.get_node("word1").await.unwrap();
    assert!(node.is_some());

    // Create user progress for that node
    let state = MemoryState::new_for_node("user1".to_string(), "word1".to_string());
    user_repo.save_memory_state(&state).await.unwrap();

    // Verify user.db has the state
    let user_state = user_repo.get_memory_state("user1", "word1").await.unwrap();
    assert!(user_state.is_some());

    // Verify app_settings table exists (migration v2 proof)
    let pool = init_user_db(":memory:").await.unwrap();
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = 'schema_version'")
        .fetch_optional(&pool)
        .await
        .unwrap();

    assert!(row.is_some(), "Migration v2 should have run");
}
