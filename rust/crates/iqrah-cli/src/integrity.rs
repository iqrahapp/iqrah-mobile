//! Database integrity checking commands

use anyhow::Result;
use iqrah_core::ContentRepository;
use iqrah_storage::{
    create_content_repository, init_content_db, init_user_db, SqliteUserRepository,
};
use std::sync::Arc;

/// Check database integrity - finds orphaned user records
pub async fn check_integrity(verbose: bool) -> Result<()> {
    println!("🔍 Checking database integrity...\n");

    // Get database paths from environment or use defaults
    let content_db_path =
        std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "data/content.db".to_string());
    let user_db_path = std::env::var("USER_DB_PATH").unwrap_or_else(|_| "data/user.db".to_string());

    println!("   Content DB: {}", content_db_path);
    println!("   User DB: {}", user_db_path);
    println!();

    // Initialize database pools
    let content_pool = init_content_db(&content_db_path).await?;
    let user_pool = init_user_db(&user_db_path).await?;

    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(create_content_repository(content_pool));
    let user_repo = SqliteUserRepository::new(user_pool);

    // Get all node IDs from user memory states
    let user_id = "default"; // TODO: parameterize
    let node_ids = user_repo.get_all_node_ids(user_id).await?;

    if node_ids.is_empty() {
        println!("ℹ️  No user memory states found for user '{}'", user_id);
        return Ok(());
    }

    println!(
        "📊 Found {} unique nodes in user memory states",
        node_ids.len()
    );

    // Check each node exists in content.db
    let mut orphaned = Vec::new();
    let mut checked = 0;

    for node_id in &node_ids {
        if !content_repo.node_exists(*node_id).await? {
            orphaned.push(*node_id);
        }
        checked += 1;

        if verbose && checked % 100 == 0 {
            print!("\r   Checked {}/{} nodes...", checked, node_ids.len());
        }
    }

    if verbose {
        println!("\r   Checked {}/{} nodes      ", checked, node_ids.len());
    }

    // Report results
    println!();
    if orphaned.is_empty() {
        println!("✅ No orphaned records found. Database integrity OK.");
    } else {
        println!(
            "⚠️  Found {} orphaned records (user state references non-existent nodes):",
            orphaned.len()
        );
        for node_id in orphaned.iter().take(10) {
            println!("   - node_id: {}", node_id);
        }
        if orphaned.len() > 10 {
            println!("   ... and {} more", orphaned.len() - 10);
        }
        println!();
        println!("💡 These records reference nodes that don't exist in content.db.");
        println!("   This can happen after a content.db update with changed node IDs.");
    }

    Ok(())
}
