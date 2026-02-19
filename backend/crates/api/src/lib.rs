//! Iqrah Backend Server library.

pub mod handlers;
pub mod middleware;

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

use iqrah_backend_config::AppConfig;
use iqrah_backend_domain::{HealthResponse, ReadyResponse};
use iqrah_backend_storage::{PackRepository, SyncRepository, UserRepository, check_connection};
use sqlx::PgPool;

use handlers::auth::IdTokenVerifier;
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

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/ready", get(ready))
        .route("/v1/auth/google", post(handlers::auth::google_auth))
        .route("/v1/packs/available", get(list_packs))
        .route("/v1/packs/{id}/download", get(download_pack))
        .route("/v1/packs/{id}/manifest", get(get_manifest))
        .route("/v1/sync/push", post(sync_push))
        .route("/v1/sync/pull", post(sync_pull))
        .route("/v1/users/me", get(handlers::auth::get_me))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
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
