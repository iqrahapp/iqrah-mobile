use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use iqrah_core::domain::ReviewGrade;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

/// Create the HTTP router with all REST endpoints
pub fn create_http_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_check))
        .route("/debug/node/:node_id", get(get_node_debug))
        .route("/debug/user/:user_id/state/:node_id", get(get_user_state))
        .route("/debug/user/:user_id/state/:node_id", post(set_user_state))
        .route("/debug/user/:user_id/review", post(process_review))
        // Translator endpoints
        .route("/languages", get(get_languages))
        .route("/translators/:language_code", get(get_translators_for_language))
        .route("/translator/:translator_id", get(get_translator))
        .route("/users/:user_id/settings/translator", get(get_user_preferred_translator))
        .route("/users/:user_id/settings/translator", post(set_user_preferred_translator))
        .route("/verses/:verse_key/translations/:translator_id", get(get_verse_translation))
        // Package management endpoints (specific routes before parameterized routes)
        .route("/packages", get(list_packages))
        .route("/packages/installed", get(list_installed_packages))
        .route("/packages/installed/:package_id", post(install_package_handler))
        .route("/packages/installed/:package_id", axum::routing::delete(uninstall_package_handler))
        .route("/packages/installed/:package_id/enable", post(enable_package_handler))
        .route("/packages/installed/:package_id/disable", post(disable_package_handler))
        .route("/packages/:package_id", get(get_package_details))
        .route("/packages/:package_id", post(upsert_package_handler))
        .route("/packages/:package_id", axum::routing::delete(delete_package_handler))
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
    let memory_state = state.user_repo.get_memory_state(&user_id, &node_id).await?;

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
struct SetStateRequest {
    energy: f64,
}

/// Set the memory state for a user and node
async fn set_user_state(
    State(state): State<Arc<AppState>>,
    Path((user_id, node_id)): Path<(String, String)>,
    Json(payload): Json<SetStateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate energy is in valid range
    if !(0.0..=1.0).contains(&payload.energy) {
        return Err(AppError::InvalidInput(
            "Energy must be between 0.0 and 1.0".to_string(),
        ));
    }

    // Update the energy
    state
        .user_repo
        .update_energy(&user_id, &node_id, payload.energy)
        .await?;

    // Return the updated state
    let memory_state = state.user_repo.get_memory_state(&user_id, &node_id).await?;

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
        None => Err(AppError::Internal(anyhow::anyhow!(
            "Failed to retrieve state after update"
        ))),
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

// ========================================================================
// Translator Endpoints
// ========================================================================

/// Get all available languages
async fn get_languages(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let languages = state.content_repo.get_languages().await?;

    let response = languages
        .into_iter()
        .map(|lang| {
            json!({
                "code": lang.code,
                "english_name": lang.english_name,
                "native_name": lang.native_name,
                "direction": lang.direction,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(response))
}

/// Get all translators for a specific language
async fn get_translators_for_language(
    State(state): State<Arc<AppState>>,
    Path(language_code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let translators = state
        .content_repo
        .get_translators_for_language(&language_code)
        .await?;

    let response = translators
        .into_iter()
        .map(|t| {
            json!({
                "id": t.id,
                "slug": t.slug,
                "full_name": t.full_name,
                "language_code": t.language_code,
                "description": t.description,
                "license": t.license,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(response))
}

/// Get a specific translator by ID
async fn get_translator(
    State(state): State<Arc<AppState>>,
    Path(translator_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let translator = state
        .content_repo
        .get_translator(translator_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Translator not found: {}", translator_id)))?;

    Ok(Json(json!({
        "id": translator.id,
        "slug": translator.slug,
        "full_name": translator.full_name,
        "language_code": translator.language_code,
        "description": translator.description,
        "license": translator.license,
    })))
}

/// Get user's preferred translator ID
async fn get_user_preferred_translator(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let translator_id = state
        .user_repo
        .get_setting("preferred_translator_id")
        .await?
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1); // Default to translator_id 1

    Ok(Json(json!({
        "user_id": user_id,
        "preferred_translator_id": translator_id,
    })))
}

#[derive(Deserialize)]
struct SetTranslatorRequest {
    translator_id: i32,
}

/// Set user's preferred translator ID
async fn set_user_preferred_translator(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(payload): Json<SetTranslatorRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate that translator exists
    let translator = state
        .content_repo
        .get_translator(payload.translator_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Translator not found: {}", payload.translator_id))
        })?;

    // Save preference
    state
        .user_repo
        .set_setting("preferred_translator_id", &payload.translator_id.to_string())
        .await?;

    Ok(Json(json!({
        "user_id": user_id,
        "preferred_translator_id": translator.id,
        "translator_name": translator.full_name,
        "message": "Preference updated successfully",
    })))
}

/// Get verse translation for a specific translator
async fn get_verse_translation(
    State(state): State<Arc<AppState>>,
    Path((verse_key, translator_id)): Path<(String, i32)>,
) -> Result<impl IntoResponse, AppError> {
    let translation = state
        .content_repo
        .get_verse_translation(&verse_key, translator_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Translation not found for verse {} with translator {}",
                verse_key, translator_id
            ))
        })?;

    Ok(Json(json!({
        "verse_key": verse_key,
        "translator_id": translator_id,
        "translation": translation,
    })))
}

// ========================================================================
// Package Management Endpoints
// ========================================================================

/// List all available packages (optionally filtered)
async fn list_packages(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let packages = state
        .content_repo
        .get_available_packages(None, None)
        .await?;

    let response = packages
        .into_iter()
        .map(|p| {
            json!({
                "package_id": p.package_id,
                "package_type": p.package_type.to_string(),
                "name": p.name,
                "language_code": p.language_code,
                "author": p.author,
                "version": p.version,
                "description": p.description,
                "file_size": p.file_size,
                "download_url": p.download_url,
                "checksum": p.checksum,
                "license": p.license,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(response))
}

/// Get package details by ID
async fn get_package_details(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let package = state
        .content_repo
        .get_package(&package_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Package not found: {}", package_id)))?;

    Ok(Json(json!({
        "package_id": package.package_id,
        "package_type": package.package_type.to_string(),
        "name": package.name,
        "language_code": package.language_code,
        "author": package.author,
        "version": package.version,
        "description": package.description,
        "file_size": package.file_size,
        "download_url": package.download_url,
        "checksum": package.checksum,
        "license": package.license,
    })))
}

#[derive(Deserialize)]
struct UpsertPackageRequest {
    package_type: String,
    name: String,
    language_code: Option<String>,
    author: Option<String>,
    version: String,
    description: Option<String>,
    file_size: Option<i64>,
    download_url: Option<String>,
    checksum: Option<String>,
    license: Option<String>,
}

/// Insert or update a package
async fn upsert_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
    Json(payload): Json<UpsertPackageRequest>,
) -> Result<impl IntoResponse, AppError> {
    use iqrah_core::PackageType;
    use std::str::FromStr;

    let package_type = PackageType::from_str(&payload.package_type)
        .map_err(|e| AppError::InvalidInput(format!("Invalid package type: {}", e)))?;

    let package = iqrah_core::ContentPackage {
        package_id: package_id.clone(),
        package_type,
        name: payload.name,
        language_code: payload.language_code,
        author: payload.author,
        version: payload.version,
        description: payload.description,
        file_size: payload.file_size,
        download_url: payload.download_url,
        checksum: payload.checksum,
        license: payload.license,
    };

    state.content_repo.upsert_package(&package).await?;

    Ok(Json(json!({
        "message": "Package upserted successfully",
        "package_id": package_id,
    })))
}

/// Delete a package
async fn delete_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.content_repo.delete_package(&package_id).await?;

    Ok(Json(json!({
        "message": "Package deleted successfully",
        "package_id": package_id,
    })))
}

/// List installed packages
async fn list_installed_packages(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let packages = state.content_repo.get_installed_packages().await?;

    let response = packages
        .into_iter()
        .map(|p| {
            json!({
                "package_id": p.package_id,
                "installed_at": p.installed_at.to_rfc3339(),
                "enabled": p.enabled,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(response))
}

/// Mark a package as installed
async fn install_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Verify package exists
    state
        .content_repo
        .get_package(&package_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Package not found: {}", package_id)))?;

    state
        .content_repo
        .mark_package_installed(&package_id)
        .await?;

    Ok(Json(json!({
        "message": "Package installed successfully",
        "package_id": package_id,
    })))
}

/// Uninstall a package
async fn uninstall_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state
        .content_repo
        .mark_package_uninstalled(&package_id)
        .await?;

    Ok(Json(json!({
        "message": "Package uninstalled successfully",
        "package_id": package_id,
    })))
}

/// Enable a package
async fn enable_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.content_repo.enable_package(&package_id).await?;

    Ok(Json(json!({
        "message": "Package enabled successfully",
        "package_id": package_id,
    })))
}

/// Disable a package
async fn disable_package_handler(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.content_repo.disable_package(&package_id).await?;

    Ok(Json(json!({
        "message": "Package disabled successfully",
        "package_id": package_id,
    })))
}
