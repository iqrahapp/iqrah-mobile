//! Iqrah Backend Server library.

pub mod cache;
pub mod handlers;
pub mod middleware;

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    routing::{get, post},
};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use iqrah_backend_config::AppConfig;
use iqrah_backend_domain::{HealthResponse, ReadyResponse};
use iqrah_backend_storage::{
    PackRepository, StorageError, SyncRepository, UserRepository, check_connection,
};
use sqlx::PgPool;

use handlers::admin_packs::{add_version, list_all_packs, publish_pack, register_pack};
use handlers::auth::IdTokenVerifier;
use handlers::packs::{download_pack, get_global_manifest, get_manifest, list_packs};
use handlers::sync::{admin_recent_conflicts, sync_pull, sync_push};

use crate::cache::pack_verification_cache::PackVerificationCache;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub pack_repo: PackRepository,
    pub user_repo: UserRepository,
    pub sync_repo: SyncRepository,
    pub id_token_verifier: Arc<dyn IdTokenVerifier>,
    pub pack_cache: PackVerificationCache,
    pub config: AppConfig,
    pub start_time: Instant,
}

impl AppState {
    pub fn invalidate_pack_cache(&self, pack_version_id: i32) {
        self.pack_cache.invalidate(pack_version_id);
    }

    pub async fn add_pack_version(
        &self,
        package_id: &str,
        version: &str,
        file_path: &str,
        size_bytes: i64,
        sha256: &str,
        min_app_version: Option<&str>,
    ) -> Result<(), StorageError> {
        if let Some(active_version_id) = self.pack_repo.get_active_version_id(package_id).await? {
            self.invalidate_pack_cache(active_version_id);
        }

        self.pack_repo
            .add_version(
                package_id,
                version,
                file_path,
                size_bytes,
                sha256,
                min_app_version,
            )
            .await
    }
}

pub fn build_router(state: Arc<AppState>) -> Router {
    let auth_rate_limit = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .period(Duration::from_secs(60))
        .burst_size(10)
        .finish()
        .expect("valid auth governor config");

    let sync_push_rate_limit = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .period(Duration::from_secs(60))
        .burst_size(30)
        .finish()
        .expect("valid sync governor config");

    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/ready", get(ready))
        .route(
            "/v1/auth/google",
            post(handlers::auth::google_auth).route_layer(GovernorLayer::new(auth_rate_limit)),
        )
        .route("/v1/packs/available", get(list_packs))
        .route("/v1/packs/manifest", get(get_global_manifest))
        .route("/v1/packs/{id}/download", get(download_pack))
        .route("/v1/packs/{id}/manifest", get(get_manifest))
        .route(
            "/v1/sync/push",
            post(sync_push)
                .route_layer(DefaultBodyLimit::max(1024 * 1024))
                .route_layer(GovernorLayer::new(sync_push_rate_limit)),
        )
        .route("/v1/sync/pull", post(sync_pull))
        .route("/v1/users/me", get(handlers::auth::get_me))
        .route("/v1/admin/packs", post(register_pack))
        .route("/v1/admin/packs", get(list_all_packs))
        .route("/v1/admin/packs/{id}/versions", post(add_version))
        .route("/v1/admin/packs/{id}/publish", post(publish_pack))
        .route(
            "/v1/admin/sync/conflicts/{user_id}",
            get(admin_recent_conflicts),
        )
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
