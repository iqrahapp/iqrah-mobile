//! Admin pack management handlers.

use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::AppState;
use crate::middleware::auth::AdminApiKey;
use iqrah_backend_domain::{DomainError, PackManifestEntry, PackManifestResponse, PackType};

#[derive(Debug, Deserialize)]
pub struct RegisterPackRequest {
    pub name: String,
    pub description: String,
    pub pack_type: PackType,
}

#[derive(Debug, Serialize)]
pub struct RegisterPackResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct AddVersionResponse {
    pub version: String,
    pub sha256: String,
    pub file_size_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct PublishPackResponse {
    pub published: bool,
}

pub async fn register_pack(
    State(state): State<Arc<AppState>>,
    _admin: AdminApiKey,
    Json(req): Json<RegisterPackRequest>,
) -> Result<(axum::http::StatusCode, Json<RegisterPackResponse>), DomainError> {
    let package_id = Uuid::new_v4().to_string();
    let pack_type = serde_json::to_string(&req.pack_type)
        .map_err(|e| DomainError::Internal(anyhow::anyhow!(e)))?
        .trim_matches('"')
        .to_string();

    state
        .pack_repo
        .register_pack(
            &package_id,
            &pack_type,
            "und",
            &req.name,
            Some(req.description.as_str()),
        )
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(RegisterPackResponse { id: package_id }),
    ))
}

pub async fn add_version(
    State(state): State<Arc<AppState>>,
    _admin: AdminApiKey,
    Path(package_id): Path<String>,
    mut multipart: Multipart,
) -> Result<(axum::http::StatusCode, Json<AddVersionResponse>), DomainError> {
    let mut version: Option<String> = None;
    let mut filename = "pack.bin".to_string();
    let mut relative_path: Option<String> = None;
    let mut file_size: u64 = 0;
    let mut hasher = Sha256::new();
    let mut file_handle: Option<fs::File> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| DomainError::Validation(format!("Invalid multipart payload: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name == "version" {
            version = Some(
                field
                    .text()
                    .await
                    .map_err(|e| DomainError::Validation(format!("Invalid version field: {e}")))?,
            );
            continue;
        }

        if name == "file" {
            if let Some(given) = field.file_name() {
                filename = given.to_string();
            }

            let version_value = version.clone().ok_or_else(|| {
                DomainError::Validation(
                    "Multipart field `version` must be provided before `file`".to_string(),
                )
            })?;

            let relative = format!("{}/{}/{}", package_id, version_value, filename);
            let full_path = PathBuf::from(&state.config.pack_storage_path).join(&relative);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    DomainError::Internal(anyhow::anyhow!("Failed to create upload directory: {e}"))
                })?;
            }

            let mut out = fs::File::create(&full_path).await.map_err(|e| {
                DomainError::Internal(anyhow::anyhow!("Failed to create uploaded file: {e}"))
            })?;

            let mut upload_field = field;
            while let Some(chunk) = upload_field
                .chunk()
                .await
                .map_err(|e| DomainError::Validation(format!("Invalid file upload chunk: {e}")))?
            {
                hasher.update(&chunk);
                file_size += chunk.len() as u64;
                out.write_all(&chunk).await.map_err(|e| {
                    DomainError::Internal(anyhow::anyhow!("Failed to write uploaded file: {e}"))
                })?;
            }

            file_handle = Some(out);
            relative_path = Some(relative);
        }
    }

    if let Some(file) = file_handle.as_mut() {
        file.flush().await.map_err(|e| {
            DomainError::Internal(anyhow::anyhow!("Failed to flush uploaded file: {e}"))
        })?;
    }

    let version =
        version.ok_or_else(|| DomainError::Validation("Missing `version` field".to_string()))?;
    let relative_path =
        relative_path.ok_or_else(|| DomainError::Validation("Missing `file` field".to_string()))?;
    let sha256 = format!("{:x}", hasher.finalize());

    state
        .add_pack_version(
            &package_id,
            &version,
            &relative_path,
            i64::try_from(file_size).map_err(|e| DomainError::Internal(anyhow::anyhow!(e)))?,
            &sha256,
            None,
        )
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(AddVersionResponse {
            version,
            sha256,
            file_size_bytes: file_size,
        }),
    ))
}

pub async fn publish_pack(
    State(state): State<Arc<AppState>>,
    _admin: AdminApiKey,
    Path(package_id): Path<String>,
) -> Result<Json<PublishPackResponse>, DomainError> {
    state
        .pack_repo
        .publish_pack(&package_id)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    Ok(Json(PublishPackResponse { published: true }))
}

pub async fn list_all_packs(
    State(state): State<Arc<AppState>>,
    _admin: AdminApiKey,
) -> Result<Json<PackManifestResponse>, DomainError> {
    let packs = state
        .pack_repo
        .list_all_packs()
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

    let base_url = &state.config.base_url;
    let entries = packs
        .into_iter()
        .map(|pack| PackManifestEntry {
            id: pack.id.clone(),
            name: pack.name,
            description: pack.description,
            pack_type: pack.pack_type,
            version: pack.version,
            sha256: pack.sha256,
            file_size_bytes: pack.file_size_bytes,
            created_at: pack.created_at,
            download_url: format!("{}/v1/packs/{}/download", base_url, pack.id),
        })
        .collect();

    Ok(Json(PackManifestResponse { packs: entries }))
}
