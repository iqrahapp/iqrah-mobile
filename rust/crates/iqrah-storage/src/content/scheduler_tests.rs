/// Unit tests for scheduler v2 storage layer methods
///
/// Tests cover:
/// - get_scheduler_candidates: Query correctness and metadata handling
/// - get_prerequisite_parents: Chunking strategy with various sizes (0, 1, 500, 501, 1000+)
/// - get_goal: Goal retrieval
/// - get_nodes_for_goal: Node listing
use crate::create_content_repository;

use iqrah_core::ContentRepository;
use sqlx::{query, SqlitePool};

/// Create an in-memory test database with scheduler v2 schema
async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // Create tables
    query(
        "CREATE TABLE nodes (
            id INTEGER PRIMARY KEY,
            type TEXT NOT NULL,
            label TEXT,
            description TEXT
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

    pool
}

#[tokio::test]
async fn test_get_scheduler_candidates_empty_goal() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool);

    let candidates = repo
        .get_scheduler_candidates("nonexistent_goal")
        .await
        .expect("Should succeed with empty result");

    assert_eq!(candidates.len(), 0);
}

#[tokio::test]
async fn test_get_scheduler_candidates_with_metadata() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Insert test data
    let node_id = nid::encode_verse(1, 1);
    query("INSERT INTO nodes (id, type, label) VALUES (?, 'verse', 'Al-Fatihah 1:1')")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'foundational_score', 0.9)")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'influence_score', 0.8)")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_metadata (node_id, key, value) VALUES (?, 'difficulty_score', 0.2)")
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
        .get_scheduler_candidates("test_goal")
        .await
        .expect("Should succeed");

    assert_eq!(candidates.len(), 1);
    let node = &candidates[0];
    assert_eq!(node.id, node_id);
    assert_eq!(node.foundational_score, 0.9);
    assert_eq!(node.influence_score, 0.8);
    assert_eq!(node.difficulty_score, 0.2);
    assert_eq!(node.energy, 0.0); // Default
    assert_eq!(node.next_due_ts, 0); // Default
    assert_eq!(node.quran_order, 1001001);
}

#[tokio::test]
async fn test_get_scheduler_candidates_missing_metadata_defaults_to_zero() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Insert node without metadata
    let node_id = nid::encode_verse(1, 2);
    query("INSERT INTO nodes (id, type, label) VALUES (?, 'verse', 'Al-Fatihah 1:2')")
        .bind(node_id)
        .execute(&pool)
        .await
        .unwrap();

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
        .get_scheduler_candidates("test_goal")
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
    let pool = create_test_db().await;
    let repo = create_content_repository(pool);

    let result = repo
        .get_prerequisite_parents(&[])
        .await
        .expect("Should succeed with empty input");

    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_get_prerequisite_parents_single_node() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create prerequisite relationship: 1:1 -> 1:2 (1:1 is prerequisite for 1:2)
    let target_id = nid::encode_verse(1, 2);
    let source_id = nid::encode_verse(1, 1);
    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source_id)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = repo
        .get_prerequisite_parents(&[target_id])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    assert_eq!(result.get(&target_id).unwrap(), &vec![source_id]);
}

#[tokio::test]
async fn test_get_prerequisite_parents_multiple_parents() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create multiple prerequisites for 1:3
    let target_id = nid::encode_verse(1, 3);
    let source_id_1 = nid::encode_verse(1, 1);
    let source_id_2 = nid::encode_verse(1, 2);

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source_id_1)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source_id_2)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = repo
        .get_prerequisite_parents(&[target_id])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    let parents = result.get(&target_id).unwrap();
    assert_eq!(parents.len(), 2);
    assert!(parents.contains(&source_id_1));
    assert!(parents.contains(&source_id_2));
}

#[tokio::test]
async fn test_get_prerequisite_parents_chunking_500_nodes() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create 500 nodes with prerequisites
    let mut node_ids = Vec::new();
    use iqrah_core::domain::node_id as nid;

    for i in 1..=500 {
        // Use WORD IDs to avoid u8 overflow in verses
        let target_id = nid::encode_word((i + 1000) as i64);
        let source_id = nid::encode_word(i as i64);
        node_ids.push(target_id);

        query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
            .bind(source_id)
            .bind(target_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let result = repo
        .get_prerequisite_parents(&node_ids)
        .await
        .expect("Should succeed with 500 nodes");

    assert_eq!(result.len(), 500);
    for i in 1..=500 {
        let target_id = nid::encode_word((i + 1000) as i64);
        let source_id = nid::encode_word(i as i64);
        assert_eq!(result.get(&target_id).unwrap(), &vec![source_id]);
    }
}

#[tokio::test]
async fn test_get_prerequisite_parents_chunking_501_nodes() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create 501 nodes (tests chunking across 2 batches)
    let mut node_ids = Vec::new();
    use iqrah_core::domain::node_id as nid;

    // Note: verse IDs would overflow for 501 items, so we use WORD IDs instead
    for _i in 1..=501 {
        // Old code removed - using WORD IDs below instead
    }

    // Reset and use WORD IDs
    node_ids.clear();
    for i in 1..=501 {
        let target_id = nid::encode_word((i + 1000) as i64);
        let source_id = nid::encode_word(i as i64);
        node_ids.push(target_id);

        query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
            .bind(source_id)
            .bind(target_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let result = repo
        .get_prerequisite_parents(&node_ids)
        .await
        .expect("Should succeed with 501 nodes (2 chunks)");

    assert_eq!(result.len(), 501);
    for i in 1..=501 {
        let target_id = nid::encode_word((i + 1000) as i64);
        let source_id = nid::encode_word(i as i64);
        assert_eq!(result.get(&target_id).unwrap(), &vec![source_id]);
    }
}

#[tokio::test]
async fn test_get_prerequisite_parents_chunking_1000_nodes() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create 1000 nodes (tests chunking across 2 batches: 500 + 500)
    let mut node_ids = Vec::new();
    use iqrah_core::domain::node_id as nid;

    for i in 1..=1000 {
        let target_id = nid::encode_word((i + 2000) as i64);
        let source_id = nid::encode_word(i as i64);
        node_ids.push(target_id);

        query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
            .bind(source_id)
            .bind(target_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let result = repo
        .get_prerequisite_parents(&node_ids)
        .await
        .expect("Should succeed with 1000 nodes (2 chunks)");

    assert_eq!(result.len(), 1000);
    for i in 1..=1000 {
        let target_id = nid::encode_word((i + 2000) as i64);
        let source_id = nid::encode_word(i as i64);
        assert_eq!(result.get(&target_id).unwrap(), &vec![source_id]);
    }
}

#[tokio::test]
async fn test_get_prerequisite_parents_excludes_non_dependency_edges() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    // Create prerequisite edge (type 0) and non-dependency edge (type 1)
    let target_id = nid::encode_verse(1, 2);
    let source_id = nid::encode_verse(1, 1);
    let source_id_3 = nid::encode_verse(1, 3);

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 0)")
        .bind(source_id)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO edges (source_id, target_id, edge_type) VALUES (?, ?, 1)")
        .bind(source_id_3)
        .bind(target_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = repo
        .get_prerequisite_parents(&[target_id])
        .await
        .expect("Should succeed");

    assert_eq!(result.len(), 1);
    let parents = result.get(&target_id).unwrap();
    assert_eq!(parents.len(), 1);
    assert!(parents.contains(&source_id));
    assert!(!parents.contains(&source_id_3));
}

#[tokio::test]
async fn test_get_goal_exists() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    query("INSERT INTO goals (goal_id, goal_type, goal_group, label, description) VALUES ('memorization:surah-1', 'surah', 'memorization', 'Memorize Al-Fatihah', 'Master all verses')")
        .execute(&pool)
        .await
        .unwrap();

    let goal = repo
        .get_goal("memorization:surah-1")
        .await
        .expect("Should succeed")
        .expect("Goal should exist");

    assert_eq!(goal.goal_id, "memorization:surah-1");
    assert_eq!(goal.goal_type, "surah");
    assert_eq!(goal.goal_group, "memorization");
    assert_eq!(goal.label, "Memorize Al-Fatihah");
    assert_eq!(goal.description, Some("Master all verses".to_string()));
}

#[tokio::test]
async fn test_get_goal_not_found() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool);

    let goal = repo.get_goal("nonexistent").await.expect("Should succeed");

    assert!(goal.is_none());
}

#[tokio::test]
async fn test_get_nodes_for_goal() {
    use iqrah_core::domain::node_id as nid;

    let pool = create_test_db().await;
    let repo = create_content_repository(pool.clone());

    query("INSERT INTO goals (goal_id, goal_type, goal_group, label) VALUES ('test_goal', 'surah', 'memorization', 'Test Goal')")
        .execute(&pool)
        .await
        .unwrap();

    // Insert nodes with different priorities
    let node_id_1 = nid::encode_verse(1, 1);
    let node_id_2 = nid::encode_verse(1, 2);
    let node_id_3 = nid::encode_verse(1, 3);

    query("INSERT INTO node_goals (node_id, goal_id, priority) VALUES (?, 'test_goal', 3.0)")
        .bind(node_id_1)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_goals (node_id, goal_id, priority) VALUES (?, 'test_goal', 1.0)")
        .bind(node_id_2)
        .execute(&pool)
        .await
        .unwrap();

    query("INSERT INTO node_goals (node_id, goal_id, priority) VALUES (?, 'test_goal', 2.0)")
        .bind(node_id_3)
        .execute(&pool)
        .await
        .unwrap();

    let nodes = repo
        .get_nodes_for_goal("test_goal")
        .await
        .expect("Should succeed");

    // Should be ordered by priority DESC, then node_id ASC
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0], node_id_1); // priority 3.0
    assert_eq!(nodes[1], node_id_3); // priority 2.0
    assert_eq!(nodes[2], node_id_2); // priority 1.0
}

#[tokio::test]
async fn test_get_nodes_for_goal_empty() {
    let pool = create_test_db().await;
    let repo = create_content_repository(pool);

    let nodes = repo
        .get_nodes_for_goal("nonexistent")
        .await
        .expect("Should succeed");

    assert_eq!(nodes.len(), 0);
}
