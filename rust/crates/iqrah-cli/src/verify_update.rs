//! Verify content.db update compatibility with user progress
//!
//! This command validates that a new content.db can safely replace the old one
//! without breaking user progress. It checks that all nodes referenced in user.db
//! still exist in the new content.db.

use anyhow::Result;
use iqrah_core::domain::node_id as nid;
use iqrah_core::ContentRepository;
use iqrah_storage::{
    create_content_repository, init_content_db, init_user_db, SqliteUserRepository,
};
use std::sync::Arc;

/// Result of verifying a content.db update
#[allow(dead_code)]
pub struct VerifyResult {
    pub total_user_nodes: usize,
    pub nodes_in_new_db: usize,
    pub missing_nodes: Vec<MissingNode>,
}

pub struct MissingNode {
    pub node_id: i64,
    pub ukey: Option<String>,
}

/// Verify that a new content.db is compatible with existing user progress
pub async fn verify_update(
    old_db_path: &str,
    new_db_path: &str,
    user_db_path: &str,
    user_id: &str,
) -> Result<VerifyResult> {
    println!("🔍 Verifying content.db update compatibility...\n");

    println!("   Old content.db: {}", old_db_path);
    println!("   New content.db: {}", new_db_path);
    println!("   User DB: {}", user_db_path);
    println!("   User ID: {}", user_id);
    println!();

    // Initialize databases
    let old_content_pool = init_content_db(old_db_path).await?;
    let new_content_pool = init_content_db(new_db_path).await?;
    let user_pool = init_user_db(user_db_path).await?;

    let old_content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(old_content_pool));
    let new_content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(new_content_pool));
    let user_repo = SqliteUserRepository::new(user_pool);

    // Get all node IDs from user progress
    let node_ids = user_repo.get_all_node_ids(user_id).await?;

    if node_ids.is_empty() {
        println!("ℹ️  No user progress found. Update is safe.");
        return Ok(VerifyResult {
            total_user_nodes: 0,
            nodes_in_new_db: 0,
            missing_nodes: vec![],
        });
    }

    println!("📊 Found {} nodes with user progress", node_ids.len());
    println!();

    // Check each node exists in both old and new content.db
    let mut missing_nodes = Vec::new();
    let mut nodes_in_new = 0;
    let mut warnings = Vec::new();

    for (idx, &node_id) in node_ids.iter().enumerate() {
        let ukey = nid::to_ukey(node_id);

        // Check old DB (for sanity - should always exist)
        let in_old = old_content_repo.node_exists(node_id).await?;
        if !in_old {
            warnings.push(format!(
                "⚠️  Node {} ({}) not found in OLD content.db (orphaned data)",
                node_id,
                ukey.as_deref().unwrap_or("unknown")
            ));
        }

        // Check new DB (critical - must exist to preserve progress)
        let in_new = new_content_repo.node_exists(node_id).await?;
        if in_new {
            nodes_in_new += 1;
        } else {
            missing_nodes.push(MissingNode {
                node_id,
                ukey: ukey.clone(),
            });
        }

        // Progress indicator
        if (idx + 1) % 100 == 0 || idx == node_ids.len() - 1 {
            print!("\r   Checked {}/{} nodes...", idx + 1, node_ids.len());
        }
    }
    println!();
    println!();

    // Print warnings
    for warning in &warnings {
        println!("{}", warning);
    }
    if !warnings.is_empty() {
        println!();
    }

    // Print result
    let result = VerifyResult {
        total_user_nodes: node_ids.len(),
        nodes_in_new_db: nodes_in_new,
        missing_nodes,
    };

    if result.missing_nodes.is_empty() {
        println!(
            "✅ Update is SAFE: All {} user nodes exist in new content.db",
            result.total_user_nodes
        );
    } else {
        println!(
            "❌ Update would BREAK {} user progress records:",
            result.missing_nodes.len()
        );
        println!();
        for (i, missing) in result.missing_nodes.iter().take(10).enumerate() {
            println!(
                "   {}. {} ({})",
                i + 1,
                missing.node_id,
                missing.ukey.as_deref().unwrap_or("unknown ukey")
            );
        }
        if result.missing_nodes.len() > 10 {
            println!("   ... and {} more", result.missing_nodes.len() - 10);
        }
        println!();
        println!("💡 These nodes exist in user.db but NOT in the new content.db.");
        println!("   User progress for these nodes would be lost after update.");
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use iqrah_core::domain::node_id as nid;
    use iqrah_core::domain::MemoryState;
    use iqrah_core::UserRepository;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_verify_update_empty_user_db() {
        let tmp = TempDir::new().unwrap();

        // Create empty databases
        let content_path = tmp.path().join("content.db");
        let user_path = tmp.path().join("user.db");

        let _content_pool = init_content_db(content_path.to_str().unwrap())
            .await
            .unwrap();
        let _user_pool = init_user_db(user_path.to_str().unwrap()).await.unwrap();

        // Verify with same DB as old and new (no changes)
        let result = verify_update(
            content_path.to_str().unwrap(),
            content_path.to_str().unwrap(),
            user_path.to_str().unwrap(),
            "default",
        )
        .await
        .unwrap();

        assert_eq!(result.total_user_nodes, 0);
        assert!(result.missing_nodes.is_empty());
    }

    #[tokio::test]
    async fn test_verify_update_detects_missing_nodes() {
        let tmp = TempDir::new().unwrap();

        // Setup paths
        let old_content_path = tmp.path().join("old_content.db");
        let new_content_path = tmp.path().join("new_content.db");
        let user_path = tmp.path().join("user.db");

        // Create content DBs - nodes table is created by init_content_db
        let old_pool = init_content_db(old_content_path.to_str().unwrap())
            .await
            .unwrap();
        let new_pool = init_content_db(new_content_path.to_str().unwrap())
            .await
            .unwrap();

        // Insert nodes using raw SQL through pool
        let verse_1_1_id = nid::from_ukey("VERSE:1:1").unwrap();
        let verse_1_2_id = nid::from_ukey("VERSE:1:2").unwrap();
        let verse_1_3_id = nid::from_ukey("VERSE:1:3").unwrap();

        // Old DB has 3 nodes
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:1:1', 2)")
            .bind(verse_1_1_id)
            .execute(&old_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:1:2', 2)")
            .bind(verse_1_2_id)
            .execute(&old_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:1:3', 2)")
            .bind(verse_1_3_id)
            .execute(&old_pool)
            .await
            .unwrap();

        // New DB has only 2 nodes (VERSE:1:2 removed!)
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:1:1', 2)")
            .bind(verse_1_1_id)
            .execute(&new_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:1:3', 2)")
            .bind(verse_1_3_id)
            .execute(&new_pool)
            .await
            .unwrap();

        // Create user.db with progress on all 3 nodes
        let user_pool = init_user_db(user_path.to_str().unwrap()).await.unwrap();
        let user_repo = SqliteUserRepository::new(user_pool);

        let now = Utc::now();
        for &node_id in &[verse_1_1_id, verse_1_2_id, verse_1_3_id] {
            user_repo
                .save_memory_state(&MemoryState {
                    user_id: "test_user".to_string(),
                    node_id,
                    stability: 1.0,
                    difficulty: 0.5,
                    energy: 0.5,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 1,
                })
                .await
                .unwrap();
        }

        // Run verification
        let result = verify_update(
            old_content_path.to_str().unwrap(),
            new_content_path.to_str().unwrap(),
            user_path.to_str().unwrap(),
            "test_user",
        )
        .await
        .unwrap();

        // Should detect VERSE:1:2 is missing
        assert_eq!(result.total_user_nodes, 3);
        assert_eq!(result.nodes_in_new_db, 2);
        assert_eq!(result.missing_nodes.len(), 1);
        assert_eq!(result.missing_nodes[0].node_id, verse_1_2_id);
        assert_eq!(result.missing_nodes[0].ukey.as_deref(), Some("VERSE:1:2"));
    }

    #[tokio::test]
    async fn test_verify_update_success_all_nodes_exist() {
        let tmp = TempDir::new().unwrap();

        // Setup paths
        let old_content_path = tmp.path().join("old_content.db");
        let new_content_path = tmp.path().join("new_content.db");
        let user_path = tmp.path().join("user.db");

        // Create content DBs
        let old_pool = init_content_db(old_content_path.to_str().unwrap())
            .await
            .unwrap();
        let new_pool = init_content_db(new_content_path.to_str().unwrap())
            .await
            .unwrap();

        let verse_255_id = nid::from_ukey("VERSE:2:255").unwrap();
        let verse_256_id = nid::from_ukey("VERSE:2:256").unwrap();

        // Both DBs have same nodes
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:2:255', 2)")
            .bind(verse_255_id)
            .execute(&old_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:2:256', 2)")
            .bind(verse_256_id)
            .execute(&old_pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:2:255', 2)")
            .bind(verse_255_id)
            .execute(&new_pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO nodes (id, ukey, node_type) VALUES (?, 'VERSE:2:256', 2)")
            .bind(verse_256_id)
            .execute(&new_pool)
            .await
            .unwrap();

        // Create user.db with progress on 2 nodes
        let user_pool = init_user_db(user_path.to_str().unwrap()).await.unwrap();
        let user_repo = SqliteUserRepository::new(user_pool);

        let now = Utc::now();
        for &node_id in &[verse_255_id, verse_256_id] {
            user_repo
                .save_memory_state(&MemoryState {
                    user_id: "user123".to_string(),
                    node_id,
                    stability: 2.0,
                    difficulty: 0.3,
                    energy: 0.7,
                    last_reviewed: now,
                    due_at: now,
                    review_count: 5,
                })
                .await
                .unwrap();
        }

        // Run verification
        let result = verify_update(
            old_content_path.to_str().unwrap(),
            new_content_path.to_str().unwrap(),
            user_path.to_str().unwrap(),
            "user123",
        )
        .await
        .unwrap();

        // All nodes should exist - update is safe
        assert_eq!(result.total_user_nodes, 2);
        assert_eq!(result.nodes_in_new_db, 2);
        assert!(result.missing_nodes.is_empty());
    }
}
