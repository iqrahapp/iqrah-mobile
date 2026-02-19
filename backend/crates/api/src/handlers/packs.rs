//! Pack API handlers.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use iqrah_backend_domain::DomainError;
use iqrah_backend_storage::PackInfo;

use crate::AppState;

/// Query parameters for pack listing.
#[derive(Debug, Deserialize)]
pub struct ListPacksQuery {
    #[serde(rename = "type")]
    pub pack_type: Option<String>,
    pub language: Option<String>,
}

/// Pack info response DTO.
#[derive(Debug, Serialize)]
pub struct PackDto {
    pub package_id: String,
    pub package_type: String,
    pub version: String,
    pub language_code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub size_bytes: i64,
    pub sha256: String,
    pub download_url: String,
}

impl PackDto {
    fn from_info(info: PackInfo, base_url: &str) -> Self {
        Self {
            package_id: info.package_id.clone(),
            package_type: info.pack_type,
            version: info.version,
            language_code: info.language,
            name: info.name,
            description: info.description,
            size_bytes: info.size_bytes,
            sha256: info.sha256,
            download_url: format!("{}/v1/packs/{}/download", base_url, info.package_id),
        }
    }
}

/// List packs response.
#[derive(Debug, Serialize)]
pub struct ListPacksResponse {
    pub packs: Vec<PackDto>,
}

/// List available packs.
pub async fn list_packs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListPacksQuery>,
) -> Result<Json<ListPacksResponse>, DomainError> {
    tracing::info!(
        pack_type = ?query.pack_type,
        language = ?query.language,
        "Listing packs"
    );

    let packs = state
        .pack_repo
        .list_available(query.pack_type.as_deref(), query.language.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Failed to list packs: {}", e);
            DomainError::Database(e.to_string())
        })?;

    let base_url = &state.config.base_url;
    let dtos: Vec<PackDto> = packs
        .into_iter()
        .map(|p| PackDto::from_info(p, base_url))
        .collect();

    tracing::info!(count = dtos.len(), "Packs listed successfully");
    Ok(Json(ListPacksResponse { packs: dtos }))
}

/// Download a pack file with range support.
pub async fn download_pack(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, DomainError> {
    tracing::info!(package_id = %package_id, "Downloading pack");

    // Get pack info
    let pack = state
        .pack_repo
        .get_pack(&package_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get pack {}: {}", package_id, e);
            DomainError::Database(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!(package_id = %package_id, "Pack not found");
            DomainError::NotFound(format!("Pack '{}' not found", package_id))
        })?;

    // Resolve file path
    let file_path = PathBuf::from(&state.config.pack_storage_path).join(&pack.file_path);

    if !file_path.exists() {
        tracing::error!("Pack file not found: {:?}", file_path);
        return Err(DomainError::NotFound(format!(
            "Pack file not found: {}",
            package_id
        )));
    }

    let mut file = File::open(&file_path).await.map_err(|e| {
        tracing::error!("Failed to open pack file: {}", e);
        DomainError::Internal(anyhow::anyhow!("Failed to open pack file: {}", e))
    })?;

    let file_size = pack.size_bytes as u64;

    // Parse Range header
    let (start, end) = parse_range_header(&headers, file_size);
    let content_length = end - start + 1;

    // Seek to start position
    if start > 0 {
        file.seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|e| {
                tracing::error!("Failed to seek: {}", e);
                DomainError::Internal(anyhow::anyhow!("Failed to seek in pack file: {}", e))
            })?;
    }

    // Create limited reader
    let limited = file.take(content_length);
    let stream = ReaderStream::new(limited);
    let body = Body::from_stream(stream);

    // Build response
    let mut response = Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, content_length)
        .header(header::ACCEPT_RANGES, "bytes")
        .header("X-Pack-SHA256", &pack.sha256);

    if start > 0 || end < file_size - 1 {
        // Partial content
        response = response.status(StatusCode::PARTIAL_CONTENT).header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end, file_size),
        );
        tracing::info!(
            package_id = %package_id,
            range = %format!("bytes {}-{}/{}", start, end, file_size),
            "Serving partial content"
        );
    } else {
        response = response.status(StatusCode::OK);
        tracing::info!(package_id = %package_id, size = file_size, "Serving full pack");
    }

    response.body(body).map_err(|e| {
        tracing::error!("Failed to build response: {}", e);
        DomainError::Internal(anyhow::anyhow!("Failed to build response: {}", e))
    })
}

/// Parse Range header, returns (start, end) byte positions.
fn parse_range_header(headers: &HeaderMap, file_size: u64) -> (u64, u64) {
    if let Some(range) = headers.get(header::RANGE)
        && let Ok(range_str) = range.to_str()
        && let Some(bytes_range) = range_str.strip_prefix("bytes=")
    {
        let parts: Vec<&str> = bytes_range.split('-').collect();
        if parts.len() == 2 {
            let start = parts[0].parse::<u64>().unwrap_or(0);
            let end = parts[1]
                .parse::<u64>()
                .unwrap_or(file_size - 1)
                .min(file_size - 1);
            return (start, end);
        }
    }
    (0, file_size - 1)
}

/// Get pack manifest only.
pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(package_id): Path<String>,
) -> Result<Json<PackDto>, DomainError> {
    tracing::info!(package_id = %package_id, "Getting pack manifest");

    let pack = state
        .pack_repo
        .get_pack(&package_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get pack {}: {}", package_id, e);
            DomainError::Database(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!(package_id = %package_id, "Pack not found");
            DomainError::NotFound(format!("Pack '{}' not found", package_id))
        })?;

    let base_url = &state.config.base_url;
    Ok(Json(PackDto::from_info(pack, base_url)))
}
