//! Iqrah Backend Server

mod handlers;
mod middleware;

use std::sync::Arc;
use std::time::Instant;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use iqrah_backend_config::AppConfig;
use iqrah_backend_domain::{HealthResponse, ReadyResponse};
use iqrah_backend_storage::{
    PackRepository, SyncRepository, UserRepository, check_connection, create_pool, run_migrations,
};
use sqlx::PgPool;

use handlers::auth::{GoogleIdTokenVerifier, IdTokenVerifier, google_auth};
use handlers::packs::{download_pack, get_manifest, list_packs};
use handlers::sync::{sync_pull, sync_push};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub pack_repo: PackRepository,
    pub user_repo: UserRepository,
    pub sync_repo: SyncRepository,
    pub id_token_verifier: Arc<dyn IdTokenVerifier>,
    pub config: AppConfig,
    pub start_time: Instant,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Iqrah Backend Server...");

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!("Binding to {}", config.bind_address);

    // Create database pool
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    run_migrations(&pool).await?;
    tracing::info!("Database migrations complete");

    // Create repositories
    let pack_repo = PackRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let sync_repo = SyncRepository::new(pool.clone());
    let id_token_verifier: Arc<dyn IdTokenVerifier> =
        Arc::new(GoogleIdTokenVerifier::new(&config.google_client_id));

    // Create app state
    let state = Arc::new(AppState {
        pool,
        pack_repo,
        user_repo,
        sync_repo,
        id_token_verifier,
        config: config.clone(),
        start_time: Instant::now(),
    });

    // Build router
    let app = Router::new()
        // Health endpoints (public)
        .route("/v1/health", get(health))
        .route("/v1/ready", get(ready))
        // Auth endpoints (public)
        .route("/v1/auth/google", post(google_auth))
        // Pack endpoints (public for now)
        .route("/v1/packs/available", get(list_packs))
        .route("/v1/packs/{id}/download", get(download_pack))
        .route("/v1/packs/{id}/manifest", get(get_manifest))
        // Sync endpoints (authenticated via AuthUser extractor)
        .route("/v1/sync/push", post(sync_push))
        .route("/v1/sync/pull", post(sync_pull))
        // User endpoints (authenticated via AuthUser extractor)
        .route("/v1/users/me", get(handlers::auth::get_me))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        // TODO: Add rate limiting middleware (tower_governor or alternative)
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    tracing::info!("Server listening on {}", config.bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint.
async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed().as_secs();

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_sha: option_env!("BUILD_SHA").unwrap_or("dev").to_string(),
        uptime_seconds: uptime,
    })
}

/// Readiness check endpoint.
async fn ready(State(state): State<Arc<AppState>>) -> Json<ReadyResponse> {
    let db_status = match check_connection(&state.pool).await {
        Ok(()) => "connected",
        Err(_) => "disconnected",
    };

    Json(ReadyResponse {
        status: if db_status == "connected" {
            "ok"
        } else {
            "degraded"
        }
        .to_string(),
        database: db_status.to_string(),
    })
}
