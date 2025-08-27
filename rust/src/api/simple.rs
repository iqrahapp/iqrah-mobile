use anyhow::Result;

use crate::repository::{MemoryState, NodeData, ReviewGrade};

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!(
        "Hello {name}! Your name has {} chars, and the reversed name is {}.",
        name.chars().count(),
        name.chars().rev().collect::<String>()
    )
}

pub fn init_database(db_path: String) -> Result<String> {
    let db_path = if db_path.is_empty() {
        None // Use in-memory for testing
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
        .get_due_items("default_user", 100)?
        .len();

    Ok(format!(
        "✅ Database initialized at {} with {} due items ready",
        db_path_str_dbg, n
    ))
}

pub fn init_database_in_memory() -> Result<String> {
    init_database(String::new())
}

pub fn get_due_items(user_id: String, limit: u32) -> Result<Vec<NodeData>> {
    crate::app::app().service.get_due_items(&user_id, limit)
}

pub fn get_node_data(node_id: String) -> Result<NodeData> {
    crate::app::app().service.get_node_data(&node_id)
}

pub fn process_review(user_id: String, node_id: String, grade: ReviewGrade) -> Result<MemoryState> {
    crate::app::app()
        .service
        .process_review(&user_id, &node_id, grade)
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}
