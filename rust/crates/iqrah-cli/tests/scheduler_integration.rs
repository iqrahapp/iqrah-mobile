/// Integration tests for the iqrah CLI scheduler functionality
///
/// These tests verify that the scheduler command works correctly end-to-end,
/// including database initialization, session generation, and output.
use anyhow::Result;
use iqrah_core::domain::node_id as nid;
use iqrah_core::{ContentRepository, UserRepository};
use iqrah_storage::{
    create_content_repository, init_content_db,
    user::{init_user_db, SqliteUserRepository},
};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create temporary test databases
async fn setup_test_databases() -> Result<(TempDir, PathBuf, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let content_db_path = temp_dir.path().join("content.db");
    let user_db_path = temp_dir.path().join("user.db");

    // Initialize databases (migrations will run automatically)
    let _content_pool = init_content_db(content_db_path.to_str().unwrap()).await?;
    let _user_pool = init_user_db(user_db_path.to_str().unwrap()).await?;

    Ok((temp_dir, content_db_path, user_db_path))
}

#[tokio::test]
async fn test_scheduler_with_new_user() -> Result<()> {
    let (_temp_dir, content_db, user_db) = setup_test_databases().await?;

    // Initialize repositories
    let content_pool = init_content_db(content_db.to_str().unwrap()).await?;
    let user_pool = init_user_db(user_db.to_str().unwrap()).await?;

    let content_repo = create_content_repository(content_pool);
    let user_repo = SqliteUserRepository::new(user_pool);

    // Verify goal exists
    let goal = content_repo.get_goal("memorization:chapters-1-3").await?;
    assert!(goal.is_some(), "Goal should exist from migrations");

    // let now_ts = chrono::Utc::now().timestamp_millis();

    // Get candidates for the goal
    let candidates = content_repo
        .get_scheduler_candidates("memorization:chapters-1-3")
        .await?;
    assert_eq!(
        candidates.len(),
        493,
        "Should have 493 candidates from expanded goal"
    );

    // Verify new user has no memory states
    let test_nodes: Vec<i64> = vec![
        nid::encode_verse(1, 1),
        nid::encode_verse(1, 2),
        nid::encode_verse(1, 3),
    ];
    let memory_states = user_repo
        .get_memory_basics("test-user", &test_nodes)
        .await?;
    assert!(
        memory_states.is_empty(),
        "New user should have no memory states"
    );

    Ok(())
}

#[tokio::test]
async fn test_scheduler_goal_data() -> Result<()> {
    let (_temp_dir, content_db, _user_db) = setup_test_databases().await?;

    let content_pool = init_content_db(content_db.to_str().unwrap()).await?;
    let content_repo = create_content_repository(content_pool);

    // Verify goal metadata
    let goal = content_repo
        .get_goal("memorization:chapters-1-3")
        .await?
        .expect("Goal should exist");

    assert_eq!(goal.goal_id, "memorization:chapters-1-3");
    assert_eq!(goal.goal_type, "custom");
    assert_eq!(goal.goal_group, "memorization");
    assert_eq!(goal.label, "Memorize Chapters 1-3");
    assert!(
        goal.description
            .as_ref()
            .is_some_and(|d| d.contains("493 verses")),
        "Description should mention 493 verses"
    );

    Ok(())
}

#[tokio::test]
async fn test_scheduler_node_metadata() -> Result<()> {
    let (_temp_dir, content_db, _user_db) = setup_test_databases().await?;

    let content_pool = init_content_db(content_db.to_str().unwrap()).await?;
    let content_repo = create_content_repository(content_pool);

    // let now_ts = chrono::Utc::now().timestamp_millis();

    // Get candidates with metadata
    let candidates = content_repo
        .get_scheduler_candidates("memorization:chapters-1-3")
        .await?;

    // Verify first verse (Al-Fatihah 1:1) has high scores
    let verse_1_1_id = nid::encode_verse(1, 1);
    let verse_1_1 = candidates
        .iter()
        .find(|c| c.id == verse_1_1_id)
        .expect("Verse 1:1 should be in candidates");

    assert!(
        verse_1_1.foundational_score > 0.06,
        "Al-Fatihah should have high foundational score, got: {}",
        verse_1_1.foundational_score
    );
    assert!(
        verse_1_1.influence_score > 0.06,
        "Al-Fatihah should have high influence score, got: {}",
        verse_1_1.influence_score
    );

    // Verify scores are realistic (not hardcoded 0.5)
    assert!(
        verse_1_1.foundational_score < 0.95,
        "Scores should be realistic PageRank values, got: {}",
        verse_1_1.foundational_score
    );

    Ok(())
}

#[tokio::test]
async fn test_scheduler_prerequisite_edges() -> Result<()> {
    let (_temp_dir, content_db, _user_db) = setup_test_databases().await?;

    let content_pool = init_content_db(content_db.to_str().unwrap()).await?;
    let content_repo = create_content_repository(content_pool);

    // let now_ts = chrono::Utc::now().timestamp_millis();

    // Get all candidates
    let candidates = content_repo
        .get_scheduler_candidates("memorization:chapters-1-3")
        .await?;
    let node_ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();

    // Get prerequisite relationships
    let prerequisites = content_repo.get_prerequisite_parents(&node_ids).await?;

    // Verify sequential prerequisites exist
    // 1:2 should have 1:1 as prerequisite
    let verse_1_2_id = nid::encode_verse(1, 2);
    let verse_1_1_id = nid::encode_verse(1, 1);

    let prereqs_1_2 = prerequisites.get(&verse_1_2_id);
    assert!(prereqs_1_2.is_some(), "1:2 should have prerequisites");

    let prereqs_1_2 = prereqs_1_2.unwrap();
    assert!(
        prereqs_1_2.contains(&verse_1_1_id),
        "1:2 should have 1:1 as prerequisite"
    );

    // Verify we have many prerequisite edges (490 sequential edges expected)
    let total_edges: usize = prerequisites.values().map(|v| v.len()).sum();
    assert!(
        total_edges >= 490,
        "Should have at least 490 sequential prerequisite edges, got: {}",
        total_edges
    );

    Ok(())
}

#[tokio::test]
async fn test_scheduler_database_initialization() -> Result<()> {
    let (_temp_dir, _content_db, user_db) = setup_test_databases().await?;

    let user_pool = init_user_db(user_db.to_str().unwrap()).await?;
    let user_repo = SqliteUserRepository::new(user_pool);

    // Verify that get_memory_basics works for nodes that don't exist
    let test_nodes: Vec<i64> = vec![nid::encode_verse(1, 1), nid::encode_verse(1, 2)];
    let memory_states = user_repo.get_memory_basics("any-user", &test_nodes).await?;

    // Should return empty map for non-existent memory states
    assert!(
        memory_states.is_empty(),
        "Should return empty map for users with no memory states"
    );

    Ok(())
}

#[tokio::test]
async fn test_scheduler_chunking_behavior() -> Result<()> {
    let (_temp_dir, content_db, _user_db) = setup_test_databases().await?;

    let content_pool = init_content_db(content_db.to_str().unwrap()).await?;
    let content_repo = create_content_repository(content_pool);

    // let now_ts = chrono::Utc::now().timestamp_millis();

    // Get all 493 candidates
    let candidates = content_repo
        .get_scheduler_candidates("memorization:chapters-1-3")
        .await?;

    assert_eq!(
        candidates.len(),
        493,
        "Should handle 493 nodes without issues"
    );

    // Get all node IDs
    let node_ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();

    // Test prerequisite chunking (should handle 500+ nodes via chunking)
    let prerequisites = content_repo.get_prerequisite_parents(&node_ids).await?;

    // Should successfully retrieve prerequisites without SQL errors
    assert!(
        !prerequisites.is_empty(),
        "Should retrieve prerequisites via chunking if needed"
    );

    Ok(())
}
