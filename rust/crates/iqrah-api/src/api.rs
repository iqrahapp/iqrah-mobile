use std::sync::Arc;
use once_cell::sync::OnceCell;
use iqrah_core::{
    ContentRepository, UserRepository,
    LearningService, SessionService, ReviewGrade,
};
use iqrah_storage::{
    SqliteContentRepository, SqliteUserRepository,
    init_content_db, init_user_db,
};

pub struct AppState {
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
}

static APP: OnceCell<AppState> = OnceCell::new();

/// Initialize the app with two databases
pub async fn init_app_async(
    content_db_path: String,
    user_db_path: String,
) -> anyhow::Result<String> {
    // Initialize content.db
    let content_pool = init_content_db(&content_db_path).await?;

    // Initialize user.db
    let user_pool = init_user_db(&user_db_path).await?;

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

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
        learning_service,
        session_service,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("App initialized with new service layer!".to_string())
}

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get().expect("App not initialized - call init_app_async first")
}

/// Process a review
pub async fn process_review_async(
    user_id: String,
    node_id: String,
    grade: u8,
) -> anyhow::Result<()> {
    let review_grade = ReviewGrade::from(grade);
    let app = app();

    app.learning_service.process_review(&user_id, &node_id, review_grade).await?;

    // Increment stats
    app.session_service.increment_stat("reviews_today").await?;

    Ok(())
}

/// Get due items for session
pub async fn get_due_items_async(
    user_id: String,
    limit: u32,
    is_high_yield: bool,
) -> anyhow::Result<Vec<DueItemDto>> {
    let app = app();

    let scored_items = app.session_service
        .get_due_items(&user_id, limit, is_high_yield)
        .await?;

    let mut result = Vec::new();
    for item in scored_items {
        result.push(DueItemDto {
            node_id: item.node.id,
            node_type: format!("{:?}", item.node.node_type),
            energy: item.memory_state.energy,
            due_at: item.memory_state.due_at.timestamp_millis(),
            priority_score: item.priority_score,
        });
    }

    Ok(result)
}

/// Get stats
pub async fn get_stats_async() -> anyhow::Result<StatsData> {
    let app = app();

    let reviews_today = app.session_service.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak = app.session_service.get_stat("streak_days").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(StatsData {
        reviews_today,
        streak_days: streak,
    })
}

/// Get due count
pub async fn get_due_count_async(user_id: String) -> anyhow::Result<u32> {
    let app = app();
    let items = app.session_service.get_due_items(&user_id, 1000, false).await?;
    Ok(items.len() as u32)
}

/// Clear session
pub async fn clear_session_async() -> anyhow::Result<String> {
    let app = app();
    app.session_service.clear_session_state().await?;
    Ok("Session cleared".to_string())
}

// DTOs for API responses
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StatsData {
    pub reviews_today: u32,
    pub streak_days: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DueItemDto {
    pub node_id: String,
    pub node_type: String,
    pub energy: f64,
    pub due_at: i64,
    pub priority_score: f64,
}

pub use StatsData as Stats;
