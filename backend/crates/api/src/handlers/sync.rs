//! Sync handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::Utc;
use validator::Validate;

use crate::AppState;
use crate::middleware::auth::{AdminApiKey, AuthUser};
use iqrah_backend_domain::{
    AdminConflictListResponse, AdminConflictRecord, DomainError, SyncPullRequest, SyncPullResponse,
    SyncPushRequest, SyncPushResponse,
};

/// Push local changes to server.
pub async fn sync_push(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<SyncPushRequest>,
) -> Result<Json<SyncPushResponse>, DomainError> {
    // Validate request
    req.validate()
        .map_err(DomainError::from_validation_errors)?;

    // Log sync operation
    tracing::info!(
        user_id = %user_id,
        device_id = %req.device_id,
        settings_count = req.changes.settings.len(),
        memory_states_count = req.changes.memory_states.len(),
        sessions_count = req.changes.sessions.len(),
        session_items_count = req.changes.session_items.len(),
        "Sync push started"
    );

    // Register device with metadata
    state
        .sync_repo
        .touch_device(
            user_id,
            req.device_id,
            req.device_os.as_deref(),
            req.device_model.as_deref(),
            req.app_version.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to touch device: {}", e);
            DomainError::Database(e.to_string())
        })?;

    // Apply changes with LWW; get applied/skipped counts
    let (applied, skipped) = state
        .sync_repo
        .apply_changes(user_id, req.device_id, &req.changes)
        .await
        .map_err(|e| {
            tracing::error!("Failed to apply changes: {}", e);
            DomainError::Database(e.to_string())
        })?;

    let server_time = Utc::now().timestamp_millis();

    tracing::info!(
        user_id = %user_id,
        device_id = %req.device_id,
        applied,
        skipped,
        server_time,
        "Sync push completed"
    );

    Ok(Json(SyncPushResponse {
        applied,
        skipped,
        server_time,
    }))
}

/// Pull changes from server since timestamp.
pub async fn sync_pull(
    State(state): State<Arc<AppState>>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<SyncPullRequest>,
) -> Result<Json<SyncPullResponse>, DomainError> {
    // Validate request
    req.validate()
        .map_err(DomainError::from_validation_errors)?;

    tracing::info!(
        user_id = %user_id,
        device_id = %req.device_id,
        since = req.since,
        "Sync pull started"
    );

    // Register device with metadata
    state
        .sync_repo
        .touch_device(
            user_id,
            req.device_id,
            req.device_os.as_deref(),
            req.device_model.as_deref(),
            req.app_version.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to touch device: {}", e);
            DomainError::Database(e.to_string())
        })?;

    // Get changes since timestamp with pagination
    let limit = req.limit.unwrap_or(1000);
    let (changes, has_more, next_cursor) = state
        .sync_repo
        .get_changes_since(user_id, req.since, limit, req.cursor.as_ref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get changes: {}", e);
            DomainError::Database(e.to_string())
        })?;

    let server_time = Utc::now().timestamp_millis();

    tracing::info!(
        user_id = %user_id,
        device_id = %req.device_id,
        settings_count = changes.settings.len(),
        memory_states_count = changes.memory_states.len(),
        sessions_count = changes.sessions.len(),
        session_items_count = changes.session_items.len(),
        has_more,
        next_cursor = ?next_cursor,
        limit,
        server_time,
        "Sync pull completed"
    );

    Ok(Json(SyncPullResponse {
        server_time,
        changes,
        has_more,
        next_cursor,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct ConflictQuery {
    pub limit: Option<usize>,
}

/// Admin-only conflict inspection endpoint.
pub async fn admin_recent_conflicts(
    State(state): State<Arc<AppState>>,
    _admin: AdminApiKey,
    Path(user_id): Path<uuid::Uuid>,
    Query(query): Query<ConflictQuery>,
) -> Result<Json<AdminConflictListResponse>, DomainError> {
    let limit = query.limit.unwrap_or(50).min(200);

    let rows = state
        .sync_repo
        .list_recent_conflicts(user_id, limit)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok(Json(AdminConflictListResponse {
        conflicts: rows
            .into_iter()
            .map(|row| AdminConflictRecord {
                id: row.id,
                user_id: row.user_id,
                entity_type: row.entity_type,
                entity_key: row.entity_key,
                incoming_metadata: row.incoming_metadata,
                winning_metadata: row.winning_metadata,
                resolved_at: row.resolved_at.timestamp_millis(),
            })
            .collect(),
    }))
}
