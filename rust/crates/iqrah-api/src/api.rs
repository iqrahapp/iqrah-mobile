use std::sync::Arc;
use once_cell::sync::OnceCell;
use iqrah_core::{ContentRepository, UserRepository};
use iqrah_storage::{
    SqliteContentRepository, SqliteUserRepository,
    init_content_db, init_user_db,
};
use crate::cbor_import::import_cbor_graph;
use crate::exercises::build_exercises_from_due_items;
use crate::review::process_review as process_review_internal;
use crate::types;

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub content_pool: sqlx::SqlitePool,
    pub user_pool: sqlx::SqlitePool,
}

static APP: OnceCell<AppState> = OnceCell::new();

/// Initialize the app with two databases
pub async fn init_app_async(
    content_db_path: String,
    user_db_path: String,
) -> anyhow::Result<String> {
    // Initialize content.db
    let content_pool = init_content_db(&content_db_path).await?;

    // Initialize user.db (runs migrations v1 and v2)
    let user_pool = init_user_db(&user_db_path).await?;

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> = Arc::new(SqliteContentRepository::new(content_pool.clone()));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool.clone()));

    // Store in global state
    APP.set(AppState {
        content_repo,
        user_repo,
        content_pool,
        user_pool,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("App initialized successfully. Two-database architecture ready.".to_string())
}

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get().expect("App not initialized - call init_app first")
}

/// Setup database: init + import CBOR graph
pub async fn setup_database_async(
    content_db_path: String,
    user_db_path: String,
    kg_bytes: Vec<u8>,
) -> anyhow::Result<String> {
    // Initialize
    init_app_async(content_db_path, user_db_path).await?;

    let app = app();

    // Check if already imported
    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&app.content_pool)
        .await?;

    if node_count == 0 {
        // Import the graph
        let stats = import_cbor_graph(&app.content_pool, kg_bytes).await?;

        Ok(format!(
            "Setup complete. Imported {} nodes and {} edges.",
            stats.nodes_imported, stats.edges_imported
        ))
    } else {
        Ok(format!(
            "Setup complete. Re-used existing DB ({} nodes)",
            node_count
        ))
    }
}

/// Get exercises for a session
pub async fn get_exercises_async(
    user_id: String,
    limit: u32,
    surah_filter: Option<i32>,
    _is_high_yield_mode: bool,
) -> anyhow::Result<Vec<types::Exercise>> {
    let app = app();
    let now = chrono::Utc::now();

    // Get due memory states
    let mut states = app.user_repo.get_due_states(&user_id, now, limit * 2).await?;

    // Filter by surah if requested
    if let Some(surah) = surah_filter {
        states.retain(|state| {
            // Check if node belongs to surah (parse node_id)
            state.node_id.starts_with(&format!("WORD_INSTANCE:{}:", surah))
        });
    }

    // Limit to requested number
    states.truncate(limit as usize);

    // Get metadata for each node
    let mut node_ids = Vec::new();
    let mut metadata_list = Vec::new();

    for state in &states {
        node_ids.push(state.node_id.clone());
        let metadata = app.content_repo.get_all_metadata(&state.node_id).await?;
        metadata_list.push(metadata);
    }

    // Build exercises
    let exercises = build_exercises_from_due_items(
        app.content_repo.as_ref(),
        app.user_repo.as_ref(),
        node_ids.clone(),
        metadata_list,
    ).await;

    // Save session state
    app.user_repo.save_session_state(&node_ids).await?;

    Ok(exercises)
}

/// Process a review
pub async fn process_review_async(
    user_id: String,
    node_id: String,
    grade: u8,
) -> anyhow::Result<String> {
    let app = app();
    let grade = types::ReviewGrade::from(grade);

    process_review_internal(
        app.content_repo.as_ref(),
        app.user_repo.as_ref(),
        &user_id,
        &node_id,
        grade,
    ).await?;

    Ok("Review processed successfully".to_string())
}

/// Get dashboard stats
pub async fn get_dashboard_stats_async(user_id: String) -> anyhow::Result<types::DashboardStats> {
    let app = app();

    let reviews_today = app.user_repo.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak_days = app.user_repo.get_stat("streak_days").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let total_reviews = app.user_repo.get_stat("total_reviews").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let due_count = app.user_repo.get_due_states(&user_id, chrono::Utc::now(), 1000).await?
        .len() as u32;

    Ok(types::DashboardStats {
        reviews_today,
        streak_days,
        due_count,
        total_reviews,
    })
}

/// Get debug stats
pub async fn get_debug_stats_async(_user_id: String) -> anyhow::Result<types::DebugStats> {
    let app = app();

    let total_nodes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes")
        .fetch_one(&app.content_pool)
        .await?;

    let total_edges_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM edges")
        .fetch_one(&app.content_pool)
        .await?;

    let user_memory_states_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_memory_states")
        .fetch_one(&app.user_pool)
        .await?;

    let due_now_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_memory_states WHERE due_at <= ?"
    )
    .bind(chrono::Utc::now().timestamp_millis())
    .fetch_one(&app.user_pool)
    .await?;

    Ok(types::DebugStats {
        total_nodes_count: total_nodes_count as u32,
        total_edges_count: total_edges_count as u32,
        user_memory_states_count: user_memory_states_count as u32,
        due_now_count: due_now_count as u32,
    })
}

/// Clear session
pub async fn clear_session_async() -> anyhow::Result<String> {
    let app = app();
    app.user_repo.clear_session_state().await?;
    Ok("Session cleared".to_string())
}

/// Get existing session
pub async fn get_existing_session_async() -> anyhow::Result<Option<Vec<types::NodeData>>> {
    let app = app();
    let node_ids = app.user_repo.get_session_state().await?;

    if node_ids.is_empty() {
        return Ok(None);
    }

    let mut nodes = Vec::new();
    for node_id in node_ids {
        // Get node metadata
        let metadata = app.content_repo.get_all_metadata(&node_id).await?;

        // Get memory state (if exists)
        let (energy, due_at) = if let Some(state) = app.user_repo.get_memory_state("default_user", &node_id).await? {
            (state.energy, state.due_at.timestamp_millis())
        } else {
            (0.0, 0)
        };

        nodes.push(types::NodeData {
            id: node_id,
            node_type: "word_instance".to_string(), // Default for now
            arabic: metadata.get("arabic").cloned(),
            translation: metadata.get("translation").cloned(),
            energy,
            due_at,
        });
    }

    Ok(Some(nodes))
}

// Re-export types
pub use crate::types::{
    Exercise, DashboardStats, DebugStats, ImportStats,
    SurahInfo, NodeData, ReviewGrade,
};
