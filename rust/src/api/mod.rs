pub mod simple;
pub mod types;

pub use simple::query_propagation_details;
pub use types::{PropagationDetailSummary, PropagationFilter};

use crate::exercises::Exercise;
use crate::repository::{DashboardStats, DebugStats, ItemPreview, MemoryState, NodeData, ReviewGrade};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Once;

static LOG_INIT: Once = Once::new();

/// One-time setup: initializes DB, imports graph, and syncs the default user.
/// Should be called on first app launch.
pub async fn setup_database(db_path: Option<String>, kg_bytes: Vec<u8>) -> Result<String> {
    // Determine paths for content.db and user.db
    let (content_path, user_path) = if let Some(base_path) = db_path {
        if base_path.is_empty() {
            (":memory:".to_string(), ":memory:".to_string())
        } else {
            let base = std::path::PathBuf::from(&base_path);
            let parent = base.parent().unwrap_or(std::path::Path::new("."));
            let content = parent.join("content.db");
            let user = parent.join("user.db");
            (
                content.to_string_lossy().to_string(),
                user.to_string_lossy().to_string(),
            )
        }
    } else {
        (":memory:".to_string(), ":memory:".to_string())
    };

    // Call new two-database setup
    iqrah_api::setup_database_async(content_path, user_path, kg_bytes).await
}

pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(None, kg_bytes).await
}

pub async fn get_exercises(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    is_high_yield_mode: bool,
) -> Result<Vec<Exercise>> {
    // Call new API
    let new_exercises = iqrah_api::get_exercises_async(user_id, limit, surah_filter, is_high_yield_mode).await?;

    // Convert from new Exercise type to old Exercise type
    let mut converted = Vec::new();
    for ex in new_exercises {
        match ex {
            iqrah_api::types::Exercise::Recall { node_id, arabic, translation } => {
                converted.push(Exercise::Recall {
                    node_id,
                    arabic,
                    translation,
                });
            }
            iqrah_api::types::Exercise::Cloze { node_id, question, answer } => {
                converted.push(Exercise::Cloze {
                    node_id,
                    question,
                    answer,
                });
            }
            iqrah_api::types::Exercise::McqArToEn {
                node_id,
                arabic,
                verse_arabic,
                surah_number,
                ayah_number,
                word_index,
                choices_en,
                correct_index,
            } => {
                converted.push(Exercise::McqArToEn {
                    node_id,
                    arabic,
                    verse_arabic,
                    surah_number,
                    ayah_number,
                    word_index,
                    choices_en,
                    correct_index,
                });
            }
            iqrah_api::types::Exercise::McqEnToAr {
                node_id,
                english,
                verse_arabic,
                surah_number,
                ayah_number,
                word_index,
                choices_ar,
                correct_index,
            } => {
                converted.push(Exercise::McqEnToAr {
                    node_id,
                    english,
                    verse_arabic,
                    surah_number,
                    ayah_number,
                    word_index,
                    choices_ar,
                    correct_index,
                });
            }
        }
    }

    Ok(converted)
}

pub async fn process_review(
    user_id: String,
    node_id: String,
    grade: ReviewGrade,
) -> Result<MemoryState> {
    // Convert grade to u8
    let grade_u8 = match grade {
        ReviewGrade::Again => 1,
        ReviewGrade::Hard => 2,
        ReviewGrade::Good => 3,
        ReviewGrade::Easy => 4,
    };

    // Call new API
    iqrah_api::process_review_async(user_id.clone(), node_id.clone(), grade_u8).await?;

    // Get updated memory state to return (for backwards compatibility)
    // For now, return a dummy state since the new API doesn't return it
    // TODO: Update Flutter to not rely on return value
    Ok(MemoryState {
        user_id,
        node_id,
        stability: 0.0,
        difficulty: 0.0,
        elapsed_days: 0,
        scheduled_days: 1,
        reps: 1,
        lapses: 0,
        last_review: chrono::Utc::now(),
        state: 0,
    })
}

pub async fn get_debug_stats(user_id: String) -> Result<DebugStats> {
    let stats = iqrah_api::get_debug_stats_async(user_id).await?;

    Ok(DebugStats {
        total_nodes_count: stats.total_nodes_count,
        total_edges_count: stats.total_edges_count,
        user_memory_states_count: stats.user_memory_states_count,
        due_now_count: stats.due_now_count,
    })
}

pub async fn reseed_database() -> Result<String> {
    // Not implemented in new API yet - would need to clear user.db
    Ok("Reseed not implemented in new architecture - user data is in separate user.db".to_string())
}

pub async fn refresh_priority_scores(user_id: String) -> Result<String> {
    // Not applicable in new architecture (FSRS handles scheduling)
    let _ = user_id;
    Ok("Priority scores handled by FSRS in new architecture".to_string())
}

pub async fn get_session_preview(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    is_high_yield_mode: bool,
) -> Result<Vec<ItemPreview>> {
    // Not implemented in new API yet - return empty for now
    let _ = (user_id, limit, surah_filter, is_high_yield_mode);
    Ok(vec![])
}

/// Search node IDs by prefix (used for sandbox suggestions)
pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeData>> {
    // Not implemented in new API yet - return empty for now
    let _ = (query, limit);
    Ok(vec![])
}

/// Fetch a single node with its metadata by ID
pub async fn fetch_node_with_metadata(node_id: String) -> Result<Option<NodeData>> {
    // Not implemented in new API yet - return None for now
    let _ = node_id;
    Ok(None)
}

/// Get existing session if one exists
pub async fn get_existing_session() -> Result<Option<Vec<NodeData>>> {
    iqrah_api::get_existing_session_async().await
}

/// Get dashboard stats (reviews today, streak)
pub async fn get_dashboard_stats(user_id: String) -> Result<DashboardStats> {
    let stats = iqrah_api::get_dashboard_stats_async(user_id).await?;

    Ok(DashboardStats {
        reviews_today: stats.reviews_today,
        streak_days: stats.streak_days,
        due_count: stats.due_count,
        total_reviews: stats.total_reviews,
    })
}

/// Clear the current session
pub async fn clear_session() -> Result<String> {
    iqrah_api::clear_session_async().await
}

// Build exercises for a specific node id (sandbox)
pub async fn get_exercises_for_node(node_id: String) -> Result<Vec<Exercise>> {
    // Not implemented in new API yet - return empty for now
    let _ = node_id;
    Ok(vec![])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
}

pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    // Not implemented in new API yet - return default surahs
    Ok((1..=114)
        .map(|i| SurahInfo {
            number: i,
            name: format!("Surah {}", i),
        })
        .collect())
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    LOG_INIT.call_once(|| {
        if tracing_subscriber::fmt::try_init().is_err() {
            tracing::debug!("tracing subscriber already initialized");
        }
    });
    flutter_rust_bridge::setup_default_user_utils();
}
