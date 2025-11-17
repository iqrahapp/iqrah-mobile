use axum::{routing::get, Router};
use iqrah_core::{
    ports::{ContentRepository, UserRepository},
    services::{LearningService, SessionService},
    ExerciseService,
};
use iqrah_storage::{
    content::{init_content_db, SqliteContentRepository},
    user::{init_user_db, SqliteUserRepository},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod http;
mod protocol;
mod websocket;

/// Application state shared across all handlers
pub struct AppState {
    pub content_repo: Arc<dyn ContentRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub learning_service: Arc<LearningService>,
    pub session_service: Arc<SessionService>,
    pub exercise_service: Arc<ExerciseService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting Iqrah Headless Test Server...");

    // Get database paths from environment or use defaults
    let content_db_path =
        std::env::var("CONTENT_DB_PATH").unwrap_or_else(|_| "data/content.db".to_string());
    let user_db_path = std::env::var("USER_DB_PATH").unwrap_or_else(|_| "data/user.db".to_string());

    tracing::info!("Content DB: {}", content_db_path);
    tracing::info!("User DB: {}", user_db_path);

    // Initialize databases
    tracing::info!("Initializing databases...");
    let content_pool = init_content_db(&content_db_path).await?;
    let user_pool = init_user_db(&user_db_path).await?;

    // Create repositories
    let content_repo: Arc<dyn ContentRepository> =
        Arc::new(SqliteContentRepository::new(content_pool));
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

    let exercise_service = Arc::new(ExerciseService::new(Arc::clone(&content_repo)));

    // Initialize semantic grading model
    tracing::info!("Initializing semantic grading model...");
    let model_path = std::env::var("SEMANTIC_MODEL_PATH")
        .unwrap_or_else(|_| "minishlab/potion-base-8M".to_string());

    match ExerciseService::init_semantic_model(&model_path) {
        Ok(_) => tracing::info!("✅ Semantic grading model initialized successfully"),
        Err(e) => {
            tracing::error!("❌ Failed to initialize semantic model: {}", e);
            tracing::error!("Model path: {}", model_path);
            tracing::error!("Set SEMANTIC_MODEL_PATH environment variable to use a different model");
            return Err(e);
        }
    }

    // Create app state
    let app_state = Arc::new(AppState {
        content_repo,
        user_repo,
        learning_service,
        session_service,
        exercise_service,
    });

    // Build the router
    let app = Router::new()
        .merge(http::create_http_router())
        .route("/ws", get(websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Start the server
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    tracing::info!("Server listening on {}", addr);
    tracing::info!("WebSocket endpoint: ws://{}/ws", addr);
    tracing::info!("Health check: http://{}/health", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
