use anyhow::Result;
use iqrah_core::{
    import_cbor_graph_from_bytes, ContentRepository, LearningService, ReviewGrade, SessionService,
    UserRepository,
};
use iqrah_storage::{init_content_db, init_user_db, SqliteContentRepository, SqliteUserRepository};
use once_cell::sync::OnceCell;
use std::sync::Arc;

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
}

static APP: OnceCell<AppState> = OnceCell::new();

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get()
        .expect("App not initialized - call setup_database first")
}

/// One-time setup: initializes databases and imports graph
pub async fn setup_database(
    content_db_path: String,
    user_db_path: String,
    kg_bytes: Vec<u8>,
) -> Result<String> {
    tracing::info!("Initializing databases...");

    // Initialize databases
    let content_pool = init_content_db(&content_db_path).await?;
    let user_pool = init_user_db(&user_db_path).await?;

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

    // Check if data already imported
    let all_nodes = content_repo.get_all_nodes().await?;

    if all_nodes.is_empty() && !kg_bytes.is_empty() {
        tracing::info!("Importing knowledge graph...");
        let cursor = std::io::Cursor::new(kg_bytes);
        let stats = import_cbor_graph_from_bytes(Arc::clone(&content_repo), cursor).await?;
        tracing::info!(
            "Import complete: {} nodes, {} edges",
            stats.nodes_imported,
            stats.edges_imported
        );
    } else {
        tracing::info!(
            "Database already contains {} nodes, skipping import",
            all_nodes.len()
        );
    }

    // Create services
    let learning_service = Arc::new(LearningService::new(
        Arc::clone(&content_repo),
        Arc::clone(&user_repo),
    ));

    let session_service = Arc::new(SessionService::new(
        Arc::clone(&content_repo),
        Arc::clone(&user_repo),
    ));

    // Store in global state
    APP.set(AppState {
        content_repo,
        user_repo,
        learning_service,
        session_service,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("Database setup complete".to_string())
}

/// Setup with in-memory databases (for testing)
pub async fn setup_database_in_memory(kg_bytes: Vec<u8>) -> Result<String> {
    setup_database(":memory:".to_string(), ":memory:".to_string(), kg_bytes).await
}

/// Get exercises for review session
pub async fn get_exercises(
    user_id: String,
    limit: u32,
    _surah_filter: Option<i32>,
    is_high_yield: bool,
) -> Result<Vec<ExerciseDto>> {
    let app = app();

    let items = app
        .session_service
        .get_due_items(&user_id, limit, is_high_yield)
        .await?;

    // Convert to DTOs (simplified for now - actual exercise generation would go here)
    let mut exercises = Vec::new();
    for item in items.into_iter().take(limit as usize) {
        // Get metadata
        let arabic = app
            .content_repo
            .get_quran_text(&item.node.id)
            .await?
            .unwrap_or_default();
        let translation = app
            .content_repo
            .get_translation(&item.node.id, "en")
            .await?
            .unwrap_or_default();

        exercises.push(ExerciseDto {
            node_id: item.node.id,
            question: arabic.clone(),
            answer: translation,
            node_type: format!("{:?}", item.node.node_type),
        });
    }

    Ok(exercises)
}

/// Process a review
pub async fn process_review(user_id: String, node_id: String, grade: u8) -> Result<String> {
    let review_grade = ReviewGrade::from(grade);
    let app = app();

    app.learning_service
        .process_review(&user_id, &node_id, review_grade)
        .await?;
    app.session_service.increment_stat("reviews_today").await?;

    Ok("Review processed".to_string())
}

/// Get dashboard stats
pub async fn get_dashboard_stats(user_id: String) -> Result<DashboardStatsDto> {
    let app = app();

    let reviews_today = app
        .session_service
        .get_stat("reviews_today")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak = app
        .session_service
        .get_stat("streak_days")
        .await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let due_items = app
        .session_service
        .get_due_items(&user_id, 1000, false)
        .await?;

    Ok(DashboardStatsDto {
        reviews_today,
        streak_days: streak,
        due_count: due_items.len() as u32,
    })
}

/// Get debug stats
pub async fn get_debug_stats(user_id: String) -> Result<DebugStatsDto> {
    let app = app();

    let all_nodes = app.content_repo.get_all_nodes().await?;
    let due_items = app
        .session_service
        .get_due_items(&user_id, 1000, false)
        .await?;

    Ok(DebugStatsDto {
        total_nodes_count: all_nodes.len() as u32,
        total_edges_count: 0, // TODO: add edge count method
        due_count: due_items.len() as u32,
    })
}

/// Reset user progress
pub async fn reseed_database(user_id: String) -> Result<String> {
    // TODO: Implement user progress reset
    Ok(format!(
        "User {} progress reset (not yet implemented)",
        user_id
    ))
}

/// Get session preview
pub async fn get_session_preview(
    user_id: String,
    limit: u32,
    is_high_yield: bool,
) -> Result<Vec<SessionPreviewDto>> {
    let app = app();

    let items = app
        .session_service
        .get_due_items(&user_id, limit, is_high_yield)
        .await?;

    let mut preview = Vec::new();
    for item in items {
        let arabic = app
            .content_repo
            .get_quran_text(&item.node.id)
            .await?
            .unwrap_or_default();

        preview.push(SessionPreviewDto {
            node_id: item.node.id,
            node_type: format!("{:?}", item.node.node_type),
            preview_text: arabic.chars().take(50).collect(),
            energy: item.memory_state.energy,
            priority_score: item.priority_score,
        });
    }

    Ok(preview)
}

/// Clear session
pub async fn clear_session() -> Result<String> {
    let app = app();
    app.session_service.clear_session_state().await?;
    Ok("Session cleared".to_string())
}

/// Search nodes
pub async fn search_nodes(query: String, limit: u32) -> Result<Vec<NodeSearchDto>> {
    let app = app();

    let all_nodes = app.content_repo.get_all_nodes().await?;

    // Simple prefix search
    let results: Vec<_> = all_nodes
        .into_iter()
        .filter(|n| n.id.starts_with(&query))
        .take(limit as usize)
        .collect();

    let mut dtos = Vec::new();
    for node in results {
        let arabic = app
            .content_repo
            .get_quran_text(&node.id)
            .await?
            .unwrap_or_default();
        dtos.push(NodeSearchDto {
            node_id: node.id,
            node_type: format!("{:?}", node.node_type),
            preview: arabic.chars().take(100).collect(),
        });
    }

    Ok(dtos)
}

/// Get available surahs
pub async fn get_available_surahs() -> Result<Vec<SurahInfo>> {
    // TODO: Implement surah listing from database
    // For now return empty - needs to query chapters from content.db
    Ok(Vec::new())
}

/// Initialize app (for Flutter bridge)
#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    use once_cell::sync::OnceCell;
    static LOG_INIT: OnceCell<()> = OnceCell::new();

    LOG_INIT.get_or_init(|| {
        if tracing_subscriber::fmt::try_init().is_err() {
            tracing::debug!("tracing subscriber already initialized");
        }
    });

    flutter_rust_bridge::setup_default_user_utils();
}

// DTOs for API responses
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ExerciseDto {
    pub node_id: String,
    pub question: String,
    pub answer: String,
    pub node_type: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DashboardStatsDto {
    pub reviews_today: u32,
    pub streak_days: u32,
    pub due_count: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DebugStatsDto {
    pub total_nodes_count: u32,
    pub total_edges_count: u32,
    pub due_count: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SessionPreviewDto {
    pub node_id: String,
    pub node_type: String,
    pub preview_text: String,
    pub energy: f64,
    pub priority_score: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeSearchDto {
    pub node_id: String,
    pub node_type: String,
    pub preview: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SurahInfo {
    pub number: i32,
    pub name: String,
}
