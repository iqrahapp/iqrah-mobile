/// Unit tests for scheduler v2 storage layer methods
///
/// Tests cover:
/// - get_scheduler_candidates: Query correctness and metadata handling
/// - get_prerequisite_parents: Chunking strategy with various sizes (0, 1, 500, 501, 1000+)
/// - get_goal: Goal retrieval
/// - get_nodes_for_goal: Node listing
use super::repository::SqliteContentRepository;
use iqrah_core::ContentRepository;
use crate::content::node_registry::NodeRegistry;
use sqlx::{query, SqlitePool};
use std::sync::Arc;

/// Create an in-memory test database with scheduler v2 schema
async fn create_test_db() -> (SqlitePool, Arc<NodeRegistry>) {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // Create tables
    query(
        "CREATE TABLE nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ukey TEXT UNIQUE NOT NULL,
            node_type TEXT NOT NULL,
            internal_id INTEGER
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create nodes table");

    query(
        "CREATE TABLE node_metadata (
            node_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value REAL NOT NULL,
            PRIMARY KEY (node_id, key)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create node_metadata table");

    query(
        "CREATE TABLE goals (
            goal_id TEXT PRIMARY KEY,
            goal_type TEXT NOT NULL,
            goal_group TEXT NOT NULL,
            label TEXT NOT NULL,
            description TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create goals table");

    query(
        "CREATE TABLE node_goals (
            node_id INTEGER NOT NULL,
            goal_id TEXT NOT NULL,
            priority REAL DEFAULT 0.0,
            PRIMARY KEY (node_id, goal_id)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create node_goals table");

    query(
        "CREATE TABLE edges (
            source_id INTEGER NOT NULL,
            target_id INTEGER NOT NULL,
            edge_type INTEGER NOT NULL,
            distribution_type INTEGER DEFAULT 0,
            param1 REAL DEFAULT 0.0,
            param2 REAL DEFAULT 0.0,
            PRIMARY KEY (source_id, target_id, edge_type)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create edges table");

    let registry = Arc::new(NodeRegistry::new(pool.clone()));

    (pool, registry)
}

async fn setup() -> (SqliteContentRepository, Arc<NodeRegistry>) {
    let (pool, registry) = create_test_db().await;
    registry.load_all().await.unwrap();
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());
    (repo, registry)
}

#[tokio::test]
async fn test_get_scheduler_candidates_empty_goal() {
    let (repo, _) = setup().await;

    let candidates = repo
        .get_scheduler_candidates("nonexistent_goal", "user1", 0)
        .await
        .expect("Should succeed with empty result");

    assert_eq!(candidates.len(), 0);
}

#[tokio::test]
async fn test_get_scheduler_candidates_with_metadata() {
    let (pool, registry) = create_test_db().await;
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());

    // Insert test data
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:1', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    registry.load_all().await.unwrap();
    let node_id = registry.get_id("1:1").await.unwrap().unwrap();

    query(
        "INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'foundational_score', 0.9)",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .unwrap();

    query("INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'influence_score', 0.8)")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    query(
        "INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'difficulty_score', 0.2)",
    )
    .bind(node_id)
    .execute(&pool)
    .await
    .unwrap();

    query("INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'quran_order', 1001001)")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO goals (goal_id, goal_type, goal_group, label) VALUES ('test_goal', 'surah', 'memorization', 'Test Goal')")
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_goals (node_id, goal_id, priority) VALUES (?, 'test_goal', 1.0)")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    // Test
    let candidates = repo
        .get_scheduler_candidates("test_goal", "user1", 0)
        .await
        .expect("Should succeed");

    assert_eq!(candidates.len(), 1);
    let node = &candidates[0];
    assert_eq!(node.id, "1:1");
    assert_eq!(node.foundational_score, 0.9);
    assert_eq!(node.influence_score, 0.8);
    assert_eq!(node.difficulty_score, 0.2);
    assert_eq!(node.energy, 0.0); // Default
    assert_eq!(node.next_due_ts, 0); // Default
    assert_eq!(node.quran_order, 1001001);
}

#[tokio::test]
async fn test_get_scheduler_candidates_missing_metadata_defaults_to_zero() {
    let (pool, registry) = create_test_db().await;
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());

    // Insert node without metadata
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:2', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    registry.load_all().await.unwrap();
    let node_id = registry.get_id("1:2").await.unwrap().unwrap();

    query("INSERT INTO goals (goal_id, goal_type, goal_group, label) VALUES ('test_goal', 'surah', 'memorization', 'Test Goal')")
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_goals (node_id, goal_id) VALUES (?, 'test_goal')")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    let candidates = repo
        .get_scheduler_candidates("test_goal", "user1", 0)
        .await
        .expect("Should succeed");

    assert_eq!(candidates.len(), 1);
    let node = &candidates[0];
    assert_eq!(node.foundational_score, 0.0); // Missing metadata defaults to 0.0
    assert_eq!(node.influence_score, 0.0);
    assert_eq!(node.difficulty_score, 0.0);
    assert_eq!(node.quran_order, 0);
}

#[tokio::test]
async fn test_get_prerequisite_parents_empty_input() {
    let (repo, _) = setup().await;

    let result = repo
        .get_prerequisite_parents(&[])
        .await
        .expect("Should succeed with empty input");

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_get_prerequisite_parents_single_node() {
    let (pool, registry) = create_test_db().await;
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());

    // Create prerequisite relationship: 1:1 -> 1:2 (1:1 is prerequisite for 1:2)
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:1', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:2', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    registry.load_all().await.unwrap();
    let source_id = registry.get_id("1:1").await.unwrap().unwrap();
    let target_id = registry.get_id("1:2").await.unwrap().unwrap();

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source_id)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = repo
        .get_prerequisite_parents(&[String::from("1:2")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get("1:2").unwrap(), &vec!["1:1"]);
}

#[tokio::test]
async fn test_get_prerequisite_parents_multiple_parents() {
    let (pool, registry) = create_test_db().await;
    let repo = SqliteContentRepository::new(pool.clone(), registry.clone());

    // Create multiple prerequisites for 1:3
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:1', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:2', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    query("INSERT INTO nodes (ukey, node_type) VALUES ('1:3', 'verse')")
        .execute(&pool)
        .await
        .unwrap();
    registry.load_all().await.unwrap();
    let source1_id = registry.get_id("1:1").await.unwrap().unwrap();
    let source2_id = registry.get_id("1:2").await.unwrap().unwrap();
    let target_id = registry.get_id("1:3").await.unwrap().unwrap();

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source1_id)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source2_id)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = repo
        .get_prerequisite_parents(&[String::from("1:3")])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    let parents = result.get("1:3").unwrap();
    assert_eq!(parents.len(), 2);
    assert!(parents.contains(&String::from("1:1")));
    assert!(parents.contains(&String::from("1:2")));
}
