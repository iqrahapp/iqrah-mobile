//! Iqrah Backend Server

use std::sync::Arc;
use std::time::Instant;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use iqrah_backend_api::actors::pack_cache::PackCacheActor;
use iqrah_backend_api::handlers::auth::{GoogleIdTokenVerifier, IdTokenVerifier};
use iqrah_backend_api::{AppState, build_router};
use iqrah_backend_config::AppConfig;
use iqrah_backend_storage::{
    PackRepository, SyncRepository, UserRepository, create_pool, run_migrations,
};
use kameo::actor::Spawn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Iqrah Backend Server...");

    let config = AppConfig::from_env()?;
    tracing::info!("Binding to {}", config.bind_address);

    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    run_migrations(&pool).await?;
    tracing::info!("Database migrations complete");

    let pack_repo = PackRepository::new(pool.clone());
    let user_repo = UserRepository::new(pool.clone());
    let sync_repo = SyncRepository::new(pool.clone());
    let id_token_verifier: Arc<dyn IdTokenVerifier> =
        Arc::new(GoogleIdTokenVerifier::new(&config.google_client_id));

    let pack_cache = PackCacheActor::spawn(PackCacheActor::new());

    let state = Arc::new(AppState {
        pool,
        pack_repo,
        user_repo,
        sync_repo,
        id_token_verifier,
        pack_cache,
        config: config.clone(),
        start_time: Instant::now(),
    });

    // Build router from shared library surface used by integration tests.
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    tracing::info!("Server listening on {}", config.bind_address);
    axum::serve(listener, app).await?;

    Ok(())
}
