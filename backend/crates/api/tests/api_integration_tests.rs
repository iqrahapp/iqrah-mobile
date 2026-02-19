#![cfg(feature = "postgres-tests")]

use std::{str::from_utf8, sync::Arc, time::Instant};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use dashmap::DashMap;
use iqrah_backend_api::handlers::auth::IdTokenVerifier;
use iqrah_backend_api::{AppState, build_router};
use iqrah_backend_config::AppConfig;
use iqrah_backend_domain::Claims;
use iqrah_backend_storage::{PackRepository, SyncRepository, UserRepository};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
struct FakeVerifier;

impl IdTokenVerifier for FakeVerifier {
    fn verify(&self, id_token: &str) -> Result<String, String> {
        if id_token == "valid-google-token" {
            Ok("google-subject-1".to_string())
        } else {
            Err("invalid token".to_string())
        }
    }
}

fn test_state(pool: PgPool, pack_dir: String) -> Arc<AppState> {
    Arc::new(AppState {
        pool: pool.clone(),
        pack_repo: PackRepository::new(pool.clone()),
        user_repo: UserRepository::new(pool.clone()),
        sync_repo: SyncRepository::new(pool),
        id_token_verifier: Arc::new(FakeVerifier),
        verified_packs: Arc::new(DashMap::new()),
        config: AppConfig {
            database_url: "postgres://unused".to_string(),
            jwt_secret: "test-secret".to_string(),
            pack_storage_path: pack_dir,
            google_client_id: "test-client-id".to_string(),
            bind_address: "127.0.0.1:0".to_string(),
            base_url: "http://localhost:8080".to_string(),
            admin_api_key: "".to_string(),
        },
        start_time: Instant::now(),
    })
}

fn auth_header(user_id: Uuid) -> String {
    let now = 1_700_000_000u64;
    let token = encode(
        &Header::default(),
        &Claims {
            sub: user_id.to_string(),
            exp: now + 3600,
            iat: now,
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .unwrap();

    format!("Bearer {token}")
}

#[sqlx::test(migrations = "../../migrations")]
async fn auth_pack_sync_and_error_paths(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    tokio::fs::write(tmp.path().join("quran-en-v1.pack"), b"abcdefghij").await?;

    let app = build_router(test_state(pool.clone(), tmp.path().display().to_string()));

    sqlx::query(
        "INSERT INTO packs (package_id, pack_type, language, name, description, status) VALUES ($1,$2,$3,$4,$5,'published')",
    )
    .bind("quran-en")
    .bind("quran")
    .bind("en")
    .bind("English Quran")
    .bind("test pack")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO pack_versions (package_id, version, file_path, size_bytes, sha256, is_active) VALUES ($1,$2,$3,$4,$5,true)",
    )
    .bind("quran-en")
    .bind("1.0.0")
    .bind("quran-en-v1.pack")
    .bind(10_i64)
    .bind("72399361da6a7754fec986df5b956c7e9d41a7fd0d03f233663d0c074f3babc3")
    .execute(&pool)
    .await?;

    tokio::fs::write(tmp.path().join("quran-ar-v1.pack"), b"mismatch-content").await?;

    sqlx::query(
        "INSERT INTO packs (package_id, pack_type, language, name, description, status) VALUES ($1,$2,$3,$4,$5,'published')",
    )
    .bind("quran-ar")
    .bind("quran")
    .bind("ar")
    .bind("Arabic Quran")
    .bind("tampered pack")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO pack_versions (package_id, version, file_path, size_bytes, sha256, is_active) VALUES ($1,$2,$3,$4,$5,true)",
    )
    .bind("quran-ar")
    .bind("1.0.0")
    .bind("quran-ar-v1.pack")
    .bind(16_i64)
    .bind("0000000000000000000000000000000000000000000000000000000000000000")
    .execute(&pool)
    .await?;

    let auth_req = Request::builder()
        .method("POST")
        .uri("/v1/auth/google")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(
            &json!({"id_token":"valid-google-token"}),
        )?))?;
    let auth_resp = app.clone().oneshot(auth_req).await?;
    assert_eq!(auth_resp.status(), StatusCode::OK);
    let auth_body: Value =
        serde_json::from_slice(&to_bytes(auth_resp.into_body(), 1024 * 1024).await?)?;
    let user_id = Uuid::parse_str(auth_body["user_id"].as_str().unwrap())?;
    let access_token = auth_body["access_token"].as_str().unwrap().to_string();

    let me_req = Request::builder()
        .uri("/v1/users/me")
        .header(header::AUTHORIZATION, format!("Bearer {access_token}"))
        .body(Body::empty())?;
    let me_resp = app.clone().oneshot(me_req).await?;
    assert_eq!(me_resp.status(), StatusCode::OK);

    let unauthorized_req = Request::builder().uri("/v1/users/me").body(Body::empty())?;
    let unauthorized_resp = app.clone().oneshot(unauthorized_req).await?;
    assert_eq!(unauthorized_resp.status(), StatusCode::UNAUTHORIZED);

    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/available?type=quran&language=en")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_json: Value =
        serde_json::from_slice(&to_bytes(list_resp.into_body(), 1024 * 1024).await?)?;
    assert_eq!(list_json["packs"].as_array().unwrap().len(), 1);

    let full_download = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/quran-en/download")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(full_download.status(), StatusCode::OK);
    assert_eq!(full_download.headers()[header::CONTENT_LENGTH], "10");
    let full_bytes = to_bytes(full_download.into_body(), 1024 * 1024).await?;
    assert_eq!(from_utf8(&full_bytes)?, "abcdefghij");

    tokio::fs::write(tmp.path().join("quran-en-v1.pack"), b"XXXXXXXXXX").await?;

    let cached_download = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/quran-en/download")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(cached_download.status(), StatusCode::OK);

    let cached_bytes = to_bytes(cached_download.into_body(), 1024 * 1024).await?;
    assert_eq!(from_utf8(&cached_bytes)?, "XXXXXXXXXX");

    let range_download = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/quran-en/download")
                .header(header::RANGE, "bytes=2-5")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(range_download.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        range_download.headers()[header::CONTENT_RANGE],
        "bytes 2-5/10"
    );
    let range_bytes = to_bytes(range_download.into_body(), 1024 * 1024).await?;
    assert_eq!(from_utf8(&range_bytes)?, "cdef");

    let tampered_pack = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/quran-ar/download")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(tampered_pack.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let tampered_json: Value =
        serde_json::from_slice(&to_bytes(tampered_pack.into_body(), 1024 * 1024).await?)?;
    assert_eq!(tampered_json["error"], "Pack integrity check failed");

    let missing_pack = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/packs/missing/download")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(missing_pack.status(), StatusCode::NOT_FOUND);

    let push_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sync/push")
                .header(header::AUTHORIZATION, auth_header(user_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&json!({
                    "device_id": Uuid::new_v4(),
                    "changes": {"settings": [{"key": "theme", "value": "dark", "client_updated_at": 1}]},
                    "device_os": "android",
                    "device_model": "pixel",
                    "app_version": "1.0.0"
                }))?))?,
        )
        .await?;
    assert_eq!(push_resp.status(), StatusCode::OK);

    let pull_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sync/pull")
                .header(header::AUTHORIZATION, auth_header(user_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(
                    &json!({"device_id": Uuid::new_v4(), "since": 0, "limit": 50}),
                )?))?,
        )
        .await?;
    assert_eq!(pull_resp.status(), StatusCode::OK);
    let pull_json: Value =
        serde_json::from_slice(&to_bytes(pull_resp.into_body(), 1024 * 1024).await?)?;
    assert_eq!(
        pull_json["changes"]["settings"].as_array().unwrap().len(),
        1
    );

    let invalid_pull = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sync/pull")
                .header(header::AUTHORIZATION, auth_header(user_id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(
                    &json!({"device_id": Uuid::new_v4(), "since": 0, "limit": 0}),
                )?))?,
        )
        .await?;
    assert_eq!(invalid_pull.status(), StatusCode::BAD_REQUEST);

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn global_manifest_returns_200_and_expected_shape(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_router(test_state(
        pool.clone(),
        tempfile::tempdir()?.path().display().to_string(),
    ));

    sqlx::query(
        "INSERT INTO packs (package_id, pack_type, language, name, description, status) VALUES ($1,$2,$3,$4,$5,'published')",
    )
    .bind("quran-fr")
    .bind("quran")
    .bind("fr")
    .bind("French Quran")
    .bind("French translation")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO pack_versions (package_id, version, file_path, size_bytes, sha256, is_active) VALUES ($1,$2,$3,$4,$5,true)",
    )
    .bind("quran-fr")
    .bind("1.5.0")
    .bind("quran-fr-v1.pack")
    .bind(777_i64)
    .bind("sha-fr")
    .execute(&pool)
    .await?;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/packs/manifest")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 1024).await?)?;
    let packs = body["packs"].as_array().expect("packs should be array");
    assert_eq!(packs.len(), 1);

    let pack = &packs[0];
    assert_eq!(pack["id"], "quran-fr");
    assert_eq!(pack["name"], "French Quran");
    assert_eq!(pack["pack_type"], "quran");
    assert_eq!(pack["version"], "1.5.0");
    assert_eq!(pack["sha256"], "sha-fr");
    assert_eq!(pack["file_size_bytes"], 777);
    assert!(pack["created_at"].is_string());
    assert_eq!(
        pack["download_url"],
        "http://localhost:8080/v1/packs/quran-fr/download"
    );

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn global_manifest_returns_empty_array_when_no_packs(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_router(test_state(
        pool,
        tempfile::tempdir()?.path().display().to_string(),
    ));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/packs/manifest")
                .body(Body::empty())?,
        )
        .await?;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 1024).await?)?;
    assert_eq!(body, json!({"packs": []}));

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn global_manifest_download_url_uses_pack_download_route(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_router(test_state(
        pool.clone(),
        tempfile::tempdir()?.path().display().to_string(),
    ));

    sqlx::query(
        "INSERT INTO packs (package_id, pack_type, language, name, description, status) VALUES ($1,$2,$3,$4,$5,'published')",
    )
    .bind("recitation-ar")
    .bind("recitation")
    .bind("ar")
    .bind("Arabic Recitation")
    .bind("recitation pack")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO pack_versions (package_id, version, file_path, size_bytes, sha256, is_active) VALUES ($1,$2,$3,$4,$5,true)",
    )
    .bind("recitation-ar")
    .bind("3.0.0")
    .bind("recitation-ar-v3.pack")
    .bind(123_i64)
    .bind("sha-rec")
    .execute(&pool)
    .await?;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/v1/packs/manifest")
                .body(Body::empty())?,
        )
        .await?;

    let body: Value = serde_json::from_slice(&to_bytes(resp.into_body(), 1024 * 1024).await?)?;
    assert_eq!(
        body["packs"][0]["download_url"],
        "http://localhost:8080/v1/packs/recitation-ar/download"
    );

    Ok(())
}
