#![cfg(feature = "postgres-tests")]

use sqlx::PgPool;

use iqrah_backend_storage::PackRepository;

#[sqlx::test(migrations = "../../migrations")]
async fn list_active_pack_versions_filters_to_active_published(
    pool: PgPool,
) -> Result<(), sqlx::Error> {
    let repo = PackRepository::new(pool.clone());

    repo.register_pack("published-pack", "quran", "en", "Published Pack", None)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.add_version(
        "published-pack",
        "1.0.0",
        "published-v1.pack",
        100,
        "sha-old",
        None,
    )
    .await
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.publish_pack("published-pack")
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.add_version(
        "published-pack",
        "1.1.0",
        "published-v2.pack",
        120,
        "sha-new",
        None,
    )
    .await
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    repo.register_pack("draft-pack", "quran", "en", "Draft Pack", None)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.add_version(
        "draft-pack",
        "0.1.0",
        "draft-v1.pack",
        50,
        "sha-draft",
        None,
    )
    .await
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let packs = repo
        .list_active_pack_versions()
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0].id, "published-pack");
    assert_eq!(packs[0].version, "1.1.0");

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_active_pack_versions_returns_empty_when_none(
    pool: PgPool,
) -> Result<(), sqlx::Error> {
    let repo = PackRepository::new(pool);

    let packs = repo
        .list_active_pack_versions()
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    assert!(packs.is_empty());

    Ok(())
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_active_pack_versions_returns_sha_and_version(
    pool: PgPool,
) -> Result<(), sqlx::Error> {
    let repo = PackRepository::new(pool);

    repo.register_pack("tafsir-ar", "tafsir", "ar", "Tafsir", Some("desc"))
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.add_version(
        "tafsir-ar",
        "2.3.4",
        "tafsir-ar-v234.pack",
        400,
        "sha256-value",
        None,
    )
    .await
    .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    repo.publish_pack("tafsir-ar")
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    let packs = repo
        .list_active_pack_versions()
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;

    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0].sha256, "sha256-value");
    assert_eq!(packs[0].version, "2.3.4");

    Ok(())
}
