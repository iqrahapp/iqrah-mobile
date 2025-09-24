use crate::{
    exercises::{create_exercise, Exercise},
    repository::{DebugStats, ItemPreview, MemoryState, ReviewGrade},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// One-time setup: initializes DB, imports graph, and syncs the default user.
/// Should be called on first app launch.
pub async fn setup_database(db_path: Option<String>, kg_bytes: Vec<u8>) -> Result<String> {
    let db_path = if db_path.is_none() || db_path.as_ref().unwrap().is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(db_path.unwrap()))
    };

    let default_user = "default_user";

    // 1. Initialize the app/repo with the db_path
    crate::app::init_app(db_path)?;
    let service = &crate::app::app().service;

    let debug_stats = service.get_debug_stats(default_user).await?;
    if debug_stats.total_nodes_count == 0 {
        // 2. Import the graph from the asset file
        let import_stats = service.import_cbor_graph_from_bytes(kg_bytes).await?;

        // 3. Create the default user and sync their nodes
        service.sync_user_nodes(default_user).await?;

        Ok(format!(
            "Setup complete. Imported {} nodes and {} edges.",
            import_stats.nodes_imported, import_stats.edges_imported
        ))
    } else {
        Ok(format!(
            "Setup complete. Re-used existing DB (nodes={}, edges={})",
            debug_stats.total_nodes_count, debug_stats.total_edges_count,
        ))
    }
}

pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(None, kg_bytes).await
}

pub async fn get_exercises(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
) -> Result<Vec<Exercise>> {
    let due_nodes = crate::app::app()
        .service
        .get_due_items(&user_id, limit * 2, surah_filter) // Get extra in case some fail to generate
        .await?;

    println!("debug:get_exercises: found {} nodes", due_nodes.len());

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

pub async fn refresh_priority_scores(user_id: String) -> Result<String> {
    crate::app::app()
        .service
        .refresh_all_priority_scores(&user_id)
        .await?;
    Ok("Priority scores refreshed".to_string())
}

pub async fn get_session_preview(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
) -> Result<Vec<ItemPreview>> {
    crate::app::app()
        .service
        .get_session_preview(&user_id, limit, surah_filter)
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
}

pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    let surahs = crate::app::app().service.get_available_surahs().await?;

    Ok(surahs
        .into_iter()
        .map(|(number, name)| SurahInfo { number, name })
        .collect())
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
