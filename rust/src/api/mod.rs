use crate::repository::{DebugStats, MemoryState, NodeData, ReviewGrade};
use anyhow::Result;

pub async fn init_database(db_path: String) -> Result<String> {
    let db_path = if db_path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(db_path))
    };

    let db_path_str_dbg = db_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or("<in-memory>".to_string());

    crate::app::init_app(db_path)?;

    let n = crate::app::app()
        .service
        .get_due_items("default_user", 100)
        .await?
        .len();

    Ok(format!(
        "Database initialized at {} with {} due items ready",
        db_path_str_dbg, n
    ))
}

pub async fn init_database_in_memory() -> Result<String> {
    init_database(String::new()).await
}

pub async fn get_due_items(user_id: String, limit: u32) -> Result<Vec<NodeData>> {
    crate::app::app()
        .service
        .get_due_items(&user_id, limit)
        .await
}

pub async fn get_node_data(node_id: String) -> Result<NodeData> {
    crate::app::app().service.get_node_data(&node_id).await
}

pub async fn process_review(
    user_id: String,
    node_id: String,
    grade: ReviewGrade,
) -> Result<MemoryState> {
    crate::app::app()
        .service
        .process_review(&user_id, &node_id, grade)
        .await
}

pub async fn get_debug_stats(user_id: String) -> Result<DebugStats> {
    crate::app::app().service.get_debug_stats(&user_id).await
}

pub async fn reseed_database() -> Result<String> {
    // Call the repo's seed method which deletes tables and reseeds
    crate::app::app().service.seed().await?;
    Ok("Database cleared and reseeded successfully".to_string())
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
