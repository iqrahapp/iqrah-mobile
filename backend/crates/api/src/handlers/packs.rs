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
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use iqrah_backend_domain::{DomainError, PackManifestEntry, PackManifestResponse};
use iqrah_backend_storage::PackInfo;

use crate::AppState;
use crate::actors::pack_cache::{Insert, PackCacheActor, Query as CacheQuery};
use kameo::actor::ActorRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeParseError {
    Invalid,
    Unsatisfiable,
}

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

/// Get global manifest for all published active packs.
pub async fn get_global_manifest(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PackManifestResponse>, DomainError> {
    tracing::info!("Getting global pack manifest");

    let packs = state
        .pack_repo
        .list_active_pack_versions()
        .await
        .map_err(|e| {
            tracing::error!("Failed to list active pack versions: {}", e);
            DomainError::Database(e.to_string())
        })?;

    let base_url = &state.config.base_url;
    let manifest_entries = packs
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

    Ok(Json(PackManifestResponse {
        packs: manifest_entries,
    }))
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

    if let Err(response) = verify_pack_integrity(
        &file_path,
        &package_id,
        pack.version_id,
        &pack.sha256,
        state.pack_cache.clone(),
    )
    .await
    {
        return Ok(response);
    }

    let file = File::open(&file_path).await.map_err(|e| {
        tracing::error!("Failed to open pack file: {}", e);
        DomainError::Internal(anyhow::anyhow!("Failed to open pack file: {}", e))
    })?;

    let file_size = file
        .metadata()
        .await
        .map_err(|e| {
            tracing::error!("Failed to stat pack file: {}", e);
            DomainError::Internal(anyhow::anyhow!("Failed to stat pack file: {}", e))
        })?
        .len();

    build_download_response(file, file_size, &pack.sha256, &headers, &package_id).await
}

async fn verify_pack_integrity(
    file_path: &PathBuf,
    package_id: &str,
    version_id: i32,
    expected_sha256: &str,
    pack_cache: ActorRef<PackCacheActor>,
) -> Result<(), Response> {
    let is_cached = pack_cache
        .ask(CacheQuery {
            pack_version_id: version_id,
        })
        .await
        .map_err(|err| {
            tracing::warn!(%err, version_id, "Failed to query pack cache");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to verify pack integrity"})),
            )
                .into_response()
        })?;

    if is_cached.unwrap_or(false) {
        return Ok(());
    }

    let computed_hash = compute_pack_sha256(file_path).await.map_err(|error| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": error.to_string()})),
        )
            .into_response()
    })?;

    if !computed_hash.eq_ignore_ascii_case(expected_sha256) {
        tracing::error!(
            package_id = %package_id,
            version_id,
            expected_sha256 = %expected_sha256,
            actual_sha256 = %computed_hash,
            "Pack integrity check failed"
        );

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Pack integrity check failed"})),
        )
            .into_response());
    }

    if let Err(err) = pack_cache
        .tell(Insert {
            pack_version_id: version_id,
            is_verified: true,
        })
        .await
    {
        tracing::warn!(%err, version_id, "Failed to update pack cache");
    }
    Ok(())
}

async fn compute_pack_sha256(file_path: &PathBuf) -> Result<String, DomainError> {
    let mut file = File::open(file_path).await.map_err(|e| {
        tracing::error!("Failed to open pack file for hashing: {}", e);
        DomainError::Internal(anyhow::anyhow!(
            "Failed to open pack file for hashing: {}",
            e
        ))
    })?;

    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer).await.map_err(|e| {
            tracing::error!("Failed to read pack file for hashing: {}", e);
            DomainError::Internal(anyhow::anyhow!(
                "Failed to read pack file for hashing: {}",
                e
            ))
        })?;

        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

async fn build_download_response(
    mut file: File,
    file_size: u64,
    sha256: &str,
    headers: &HeaderMap,
    package_id: &str,
) -> Result<Response, DomainError> {
    // Parse Range header
    let parsed_range = match parse_range_header(headers, file_size) {
        Ok(range) => range,
        Err(RangeParseError::Invalid | RangeParseError::Unsatisfiable) => {
            let response = Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
                .header(header::ACCEPT_RANGES, "bytes")
                .header("X-Pack-SHA256", sha256)
                .body(Body::empty())
                .map_err(|e| {
                    tracing::error!("Failed to build 416 response: {}", e);
                    DomainError::Internal(anyhow::anyhow!("Failed to build 416 response: {}", e))
                })?;
            return Ok(response);
        }
    };

    let (start, end) = parsed_range.unwrap_or_else(|| {
        if file_size == 0 {
            (0, 0)
        } else {
            (0, file_size - 1)
        }
    });
    let content_length = if file_size == 0 { 0 } else { end - start + 1 };

    // Seek to start position
    if content_length > 0 && start > 0 {
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
        .header("X-Pack-SHA256", sha256);

    if content_length > 0 && (start > 0 || end < file_size - 1) {
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
fn parse_range_header(
    headers: &HeaderMap,
    file_size: u64,
) -> Result<Option<(u64, u64)>, RangeParseError> {
    let Some(raw_range) = headers.get(header::RANGE) else {
        return Ok(None);
    };

    let range_str = raw_range.to_str().map_err(|_| RangeParseError::Invalid)?;
    let bytes_range = range_str
        .strip_prefix("bytes=")
        .ok_or(RangeParseError::Invalid)?;

    if bytes_range.contains(',') {
        return Err(RangeParseError::Invalid);
    }

    let (start_str, end_str) = bytes_range
        .split_once('-')
        .ok_or(RangeParseError::Invalid)?;

    let range = match (start_str.is_empty(), end_str.is_empty()) {
        (false, false) => {
            let start = start_str
                .parse::<u64>()
                .map_err(|_| RangeParseError::Invalid)?;
            let end = end_str
                .parse::<u64>()
                .map_err(|_| RangeParseError::Invalid)?;

            if start > end {
                return Err(RangeParseError::Invalid);
            }
            if start >= file_size {
                return Err(RangeParseError::Unsatisfiable);
            }

            (start, end.min(file_size.saturating_sub(1)))
        }
        (false, true) => {
            let start = start_str
                .parse::<u64>()
                .map_err(|_| RangeParseError::Invalid)?;
            if start >= file_size {
                return Err(RangeParseError::Unsatisfiable);
            }

            (start, file_size.saturating_sub(1))
        }
        (true, false) => {
            let suffix_len = end_str
                .parse::<u64>()
                .map_err(|_| RangeParseError::Invalid)?;
            if suffix_len == 0 || file_size == 0 {
                return Err(RangeParseError::Unsatisfiable);
            }

            let len = suffix_len.min(file_size);
            (file_size - len, file_size - 1)
        }
        (true, true) => return Err(RangeParseError::Invalid),
    };

    Ok(Some(range))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use kameo::actor::Spawn;
    use tokio::io::AsyncWriteExt;

    fn headers_with_range(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn parse_range_supports_standard_forms() {
        let headers = headers_with_range("bytes=10-20");
        assert_eq!(parse_range_header(&headers, 100), Ok(Some((10, 20))));

        let headers = headers_with_range("bytes=10-");
        assert_eq!(parse_range_header(&headers, 100), Ok(Some((10, 99))));

        let headers = headers_with_range("bytes=-15");
        assert_eq!(parse_range_header(&headers, 100), Ok(Some((85, 99))));
    }

    #[test]
    fn parse_range_caps_end_to_file_size() {
        let headers = headers_with_range("bytes=90-150");
        assert_eq!(parse_range_header(&headers, 100), Ok(Some((90, 99))));
    }

    #[test]
    fn parse_range_rejects_invalid_and_unsatisfiable_ranges() {
        assert_eq!(
            parse_range_header(&headers_with_range("bytes=20-10"), 100),
            Err(RangeParseError::Invalid)
        );
        assert_eq!(
            parse_range_header(&headers_with_range("bytes=100-120"), 100),
            Err(RangeParseError::Unsatisfiable)
        );
        assert_eq!(
            parse_range_header(&headers_with_range("bytes=-0"), 100),
            Err(RangeParseError::Unsatisfiable)
        );
        assert_eq!(
            parse_range_header(&headers_with_range("bytes=0-1,4-5"), 100),
            Err(RangeParseError::Invalid)
        );
    }

    #[test]
    fn parse_range_without_header_is_none() {
        assert_eq!(parse_range_header(&HeaderMap::new(), 100), Ok(None));
    }

    async fn write_temp_file(content: &[u8]) -> tempfile::NamedTempFile {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let mut file = File::create(temp.path()).await.unwrap();
        file.write_all(content).await.unwrap();
        temp
    }

    #[tokio::test]
    async fn verify_pack_integrity_accepts_matching_hash() {
        let temp = write_temp_file(b"abc").await;
        let cache = PackCacheActor::spawn(PackCacheActor::new());

        let result = verify_pack_integrity(
            &temp.path().to_path_buf(),
            "pkg",
            1,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            cache.clone(),
        )
        .await;

        assert!(result.is_ok());
        let cached = cache.ask(CacheQuery { pack_version_id: 1 }).await.unwrap();
        assert_eq!(cached, Some(true));
    }

    #[tokio::test]
    async fn verify_pack_integrity_rejects_tampered_file() {
        let temp = write_temp_file(b"abc").await;
        let cache = PackCacheActor::spawn(PackCacheActor::new());

        let response = verify_pack_integrity(
            &temp.path().to_path_buf(),
            "pkg",
            2,
            "0000000000000000000000000000000000000000000000000000000000000000",
            cache.clone(),
        )
        .await
        .unwrap_err();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn verify_pack_integrity_uses_cache_on_second_request() {
        let temp = write_temp_file(b"abc").await;
        let path = temp.path().to_path_buf();
        let cache = PackCacheActor::spawn(PackCacheActor::new());

        verify_pack_integrity(
            &path,
            "pkg",
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            cache.clone(),
        )
        .await
        .unwrap();

        tokio::fs::write(&path, b"mutated").await.unwrap();

        let second = verify_pack_integrity(
            &path,
            "pkg",
            3,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            cache.clone(),
        )
        .await;

        assert!(second.is_ok());
    }

    #[tokio::test]
    async fn download_response_full_request_returns_200_with_full_body() {
        let temp = write_temp_file(b"abcdefghijklmnopqrstuvwxyz").await;
        let file = File::open(temp.path()).await.unwrap();

        let response = build_download_response(file, 26, "abc123", &HeaderMap::new(), "pkg")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::ACCEPT_RANGES], "bytes");
        assert_eq!(response.headers()["X-Pack-SHA256"], "abc123");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"abcdefghijklmnopqrstuvwxyz");
    }

    #[tokio::test]
    async fn download_response_zero_length_file_is_safe() {
        let temp = write_temp_file(b"").await;
        let file = File::open(temp.path()).await.unwrap();

        let response = build_download_response(file, 0, "abc123", &HeaderMap::new(), "pkg")
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "0");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.is_empty());
    }

    #[tokio::test]
    async fn download_response_partial_request_returns_206() {
        let temp = write_temp_file(b"abcdefghijklmnopqrstuvwxyz").await;
        let file = File::open(temp.path()).await.unwrap();

        let response =
            build_download_response(file, 26, "abc123", &headers_with_range("bytes=5-9"), "pkg")
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes 5-9/26");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"fghij");
    }

    #[tokio::test]
    async fn download_response_invalid_range_returns_416() {
        let temp = write_temp_file(b"abcdefghijklmnopqrstuvwxyz").await;
        let file = File::open(temp.path()).await.unwrap();

        let response = build_download_response(
            file,
            26,
            "abc123",
            &headers_with_range("bytes=30-35"),
            "pkg",
        )
        .await
        .unwrap();

        assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
        assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes */26");

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(body.is_empty());
    }
}
