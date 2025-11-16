use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use iqrah_core::domain::{MemoryState, ReviewGrade};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

/// Create the HTTP router with all REST endpoints
pub fn create_http_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/debug/node/:node_id", get(get_node_debug))
        .route("/debug/user/:user_id/state/:node_id", get(get_user_state))
        .route("/debug/user/:user_id/review", post(process_review))
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

/// Get all available metadata for a node
async fn get_node_debug(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Get the node
    let node = state
        .content_repo
        .get_node(&node_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Node not found: {}", node_id)))?;

    // Get Quran text if available
    let quran_text = state.content_repo.get_quran_text(&node_id).await.ok();

    // Get translation (default to English)
    let translation = state
        .content_repo
        .get_translation(&node_id, "en")
        .await
        .ok();

    // Get edges
    let edges = state.content_repo.get_edges_from(&node_id).await?;

    let response = json!({
        "node": {
            "id": node.id,
            "node_type": format!("{:?}", node.node_type),
        },
        "quran_text": quran_text.flatten(),
        "translation": translation.flatten(),
        "edges": edges.iter().map(|e| json!({
            "target_id": e.target_id,
            "edge_type": format!("{:?}", e.edge_type),
            "distribution_type": format!("{:?}", e.distribution_type),
            "param1": e.param1,
            "param2": e.param2,
        })).collect::<Vec<_>>(),
    });

    Ok(Json(response))
}

/// Get the current memory state for a user and node
async fn get_user_state(
    State(state): State<Arc<AppState>>,
    Path((user_id, node_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let memory_state = state
        .user_repo
        .get_memory_state(&user_id, &node_id)
        .await?;

    match memory_state {
        Some(state) => Ok(Json(json!({
            "user_id": state.user_id,
            "node_id": state.node_id,
            "stability": state.stability,
            "difficulty": state.difficulty,
            "energy": state.energy,
            "last_reviewed": state.last_reviewed.to_rfc3339(),
            "due_at": state.due_at.to_rfc3339(),
            "review_count": state.review_count,
        }))),
        None => Ok(Json(json!({
            "user_id": user_id,
            "node_id": node_id,
            "state": null,
            "message": "No memory state found (never reviewed)"
        }))),
    }
}

#[derive(Deserialize)]
struct ReviewRequest {
    node_id: String,
    grade: String, // "Again", "Hard", "Good", "Easy"
}

/// Process a single review
async fn process_review(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<ReviewRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Parse the grade
    let grade = parse_review_grade(&payload.grade)?;

    // Process the review
    let updated_state = state
        .learning_service
        .process_review(&user_id, &payload.node_id, grade)
        .await?;

    Ok(Json(json!({
        "user_id": updated_state.user_id,
        "node_id": updated_state.node_id,
        "stability": updated_state.stability,
        "difficulty": updated_state.difficulty,
        "energy": updated_state.energy,
        "last_reviewed": updated_state.last_reviewed.to_rfc3339(),
        "due_at": updated_state.due_at.to_rfc3339(),
        "review_count": updated_state.review_count,
    })))
}

/// Parse a review grade from string
fn parse_review_grade(grade: &str) -> Result<ReviewGrade, AppError> {
    match grade {
        "Again" => Ok(ReviewGrade::Again),
        "Hard" => Ok(ReviewGrade::Hard),
        "Good" => Ok(ReviewGrade::Good),
        "Easy" => Ok(ReviewGrade::Easy),
        _ => Err(AppError::InvalidInput(format!(
            "Invalid grade: {}. Must be one of: Again, Hard, Good, Easy",
            grade
        ))),
    }
}

/// Application error type
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InvalidInput(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", err),
            ),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
