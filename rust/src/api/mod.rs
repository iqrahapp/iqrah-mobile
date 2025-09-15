use crate::{
    exercises::{create_exercise, Exercise},
    repository::{DebugStats, MemoryState, ReviewGrade},
};
use anyhow::Result;

/// One-time setup: initializes DB, imports graph, and syncs the default user.
/// Should be called on first app launch.
pub async fn setup_database(db_path: Option<String>, kg_bytes: Vec<u8>) -> Result<String> {
    let db_path = if db_path.is_none() || db_path.as_ref().unwrap().is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(db_path.unwrap()))
    };

    // 1. Initialize the app/repo with the db_path
    crate::app::init_app(db_path)?;
    let service = &crate::app::app().service;

    // 2. Import the graph from the asset file
    let import_stats = service.import_cbor_graph_from_bytes(kg_bytes).await?;

    // 3. Create the default user and sync their nodes
    service.sync_user_nodes("default_user").await?;

    Ok(format!(
        "Setup complete. Imported {} nodes and {} edges.",
        import_stats.nodes_imported, import_stats.edges_imported
    ))
}

pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(None, kg_bytes).await
}

pub async fn get_exercises(user_id: String, limit: u32) -> Result<Vec<Exercise>> {
    let due_nodes = crate::app::app()
        .service
        .get_due_items(&user_id, limit * 2) // Get extra in case some fail to generate
        .await?;

    let exercises: Vec<Exercise> = due_nodes
        .into_iter()
        .filter_map(|node| create_exercise(node).ok()) // Skip any that fail
        .take(limit as usize) // Take only what we need after filtering
        .collect();

    Ok(exercises)
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
    crate::app::app()
        .service
        .reset_user_progress("default_user")
        .await?;
    Ok("User progress reset successfully".to_string())
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
