use std::sync::Arc;
use once_cell::sync::OnceCell;
use iqrah_core::{ContentRepository, UserRepository};
use iqrah_storage::{
    SqliteContentRepository, SqliteUserRepository,
    init_content_db, init_user_db,
};

pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
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
    let content_repo: Arc<dyn ContentRepository> = Arc::new(SqliteContentRepository::new(content_pool));
    let user_repo: Arc<dyn UserRepository> = Arc::new(SqliteUserRepository::new(user_pool));

    // Store in global state
    APP.set(AppState {
        content_repo,
        user_repo,
    })
    .map_err(|_| anyhow::anyhow!("App already initialized"))?;

    Ok("App initialized successfully. Two-database architecture ready.".to_string())
}

/// Get app state (helper function)
fn app() -> &'static AppState {
    APP.get().expect("App not initialized - call init_app first")
}

/// Simple stats for validation
pub async fn get_stats_async() -> anyhow::Result<StatsData> {
    let app = app();

    let reviews_today = app.user_repo.get_stat("reviews_today").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let streak = app.user_repo.get_stat("streak_days").await?
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(StatsData {
        reviews_today,
        streak_days: streak,
    })
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StatsData {
    pub reviews_today: u32,
    pub streak_days: u32,
}

/// Get due items count
pub async fn get_due_count_async(user_id: String) -> anyhow::Result<u32> {
    let app = app();
    let now = chrono::Utc::now();

    let states = app.user_repo.get_due_states(&user_id, now, 1000).await?;

    Ok(states.len() as u32)
}

/// Clear session state
pub async fn clear_session_async() -> anyhow::Result<String> {
    let app = app();
    app.user_repo.clear_session_state().await?;
    Ok("Session cleared".to_string())
}

// Re-export for convenience
pub use StatsData as Stats;
