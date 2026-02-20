//! Pack repository for storage layer.

use crate::StorageError;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Pack record from database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PackRow {
    pub package_id: String,
    pub pack_type: String,
    pub language: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
}

/// Pack version record from database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PackVersionRow {
    pub id: i32,
    pub package_id: String,
    pub version: String,
    pub file_path: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub min_app_version: Option<String>,
    pub published_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Combined pack info for API responses.
#[derive(Debug, Clone)]
pub struct PackInfo {
    pub version_id: i32,
    pub package_id: String,
    pub pack_type: String,
    pub version: String,
    pub language: String,
    pub name: String,
    pub description: Option<String>,
    pub size_bytes: i64,
    pub sha256: String,
    pub file_path: String,
}

/// Active pack version info for global manifest responses.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PackVersionInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub pack_type: String,
    pub version: String,
    pub sha256: String,
    pub file_size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

/// Pack repository.
#[derive(Clone)]
pub struct PackRepository {
    pool: PgPool,
}

impl PackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List available packs with optional filters.
    pub async fn list_available(
        &self,
        pack_type: Option<&str>,
        language: Option<&str>,
    ) -> Result<Vec<PackInfo>, StorageError> {
        let rows = sqlx::query_as::<_, PackInfoRow>(
            r#"
            SELECT 
                pv.id as version_id,
                p.package_id,
                p.pack_type,
                pv.version,
                p.language,
                COALESCE(p.name, p.package_id) as name,
                p.description,
                pv.size_bytes,
                pv.sha256,
                pv.file_path
            FROM packs p
            JOIN pack_versions pv ON p.package_id = pv.package_id AND pv.is_active = true
            WHERE p.status = 'published'
            AND ($1::text IS NULL OR p.pack_type = $1)
            AND ($2::text IS NULL OR p.language = $2)
            ORDER BY p.package_id
            "#,
        )
        .bind(pack_type)
        .bind(language)
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get a specific pack version.
    pub async fn get_pack(&self, package_id: &str) -> Result<Option<PackInfo>, StorageError> {
        let row = sqlx::query_as::<_, PackInfoRow>(
            r#"
            SELECT 
                pv.id as version_id,
                p.package_id,
                p.pack_type,
                pv.version,
                p.language,
                COALESCE(p.name, p.package_id) as name,
                p.description,
                pv.size_bytes,
                pv.sha256,
                pv.file_path
            FROM packs p
            JOIN pack_versions pv ON p.package_id = pv.package_id AND pv.is_active = true
            WHERE p.package_id = $1
            "#,
        )
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(row.map(|r| r.into()))
    }

    /// List all active versions for published packs.
    pub async fn list_active_pack_versions(&self) -> Result<Vec<PackVersionInfo>, StorageError> {
        sqlx::query_as::<_, PackVersionInfo>(
            r#"
            SELECT
                p.package_id AS id,
                COALESCE(p.name, p.package_id) AS name,
                p.description,
                p.pack_type,
                pv.version,
                pv.sha256,
                pv.size_bytes AS file_size_bytes,
                pv.published_at AS created_at
            FROM packs p
            JOIN pack_versions pv ON p.package_id = pv.package_id
            WHERE p.status = 'published' AND pv.is_active = true
            ORDER BY p.package_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Query)
    }

    /// List latest versions for all packs regardless of publish state.
    pub async fn list_all_packs(&self) -> Result<Vec<PackVersionInfo>, StorageError> {
        sqlx::query_as::<_, PackVersionInfo>(
            r#"
            SELECT
                p.package_id AS id,
                COALESCE(p.name, p.package_id) AS name,
                p.description,
                p.pack_type,
                pv.version,
                pv.sha256,
                pv.size_bytes AS file_size_bytes,
                pv.published_at AS created_at
            FROM packs p
            JOIN LATERAL (
                SELECT version, sha256, size_bytes, published_at
                FROM pack_versions
                WHERE package_id = p.package_id
                ORDER BY published_at DESC, id DESC
                LIMIT 1
            ) pv ON true
            ORDER BY p.package_id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Query)
    }

    /// Return the currently active version id for a pack, if present.
    pub async fn get_active_version_id(
        &self,
        package_id: &str,
    ) -> Result<Option<i32>, StorageError> {
        let row = sqlx::query_scalar::<_, i32>(
            "SELECT id FROM pack_versions WHERE package_id = $1 AND is_active = true LIMIT 1",
        )
        .bind(package_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(row)
    }

    /// Register a new pack.
    pub async fn register_pack(
        &self,
        package_id: &str,
        pack_type: &str,
        language: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO packs (package_id, pack_type, language, name, description, status)
            VALUES ($1, $2, $3, $4, $5, 'draft')
            ON CONFLICT (package_id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description
            "#,
        )
        .bind(package_id)
        .bind(pack_type)
        .bind(language)
        .bind(name)
        .bind(description)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(())
    }

    /// Add a new pack version.
    pub async fn add_version(
        &self,
        package_id: &str,
        version: &str,
        file_path: &str,
        size_bytes: i64,
        sha256: &str,
        min_app_version: Option<&str>,
    ) -> Result<(), StorageError> {
        // Deactivate previous versions
        sqlx::query("UPDATE pack_versions SET is_active = false WHERE package_id = $1")
            .bind(package_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Query)?;

        // Insert new version
        sqlx::query(
            r#"
            INSERT INTO pack_versions (package_id, version, file_path, size_bytes, sha256, min_app_version, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, true)
            "#,
        )
        .bind(package_id)
        .bind(version)
        .bind(file_path)
        .bind(size_bytes)
        .bind(sha256)
        .bind(min_app_version)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Query)?;

        Ok(())
    }

    /// Publish a pack (make it available for download).
    pub async fn publish_pack(&self, package_id: &str) -> Result<(), StorageError> {
        sqlx::query("UPDATE packs SET status = 'published' WHERE package_id = $1")
            .bind(package_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Query)?;

        Ok(())
    }
}

/// Internal row type for query mapping.
#[derive(sqlx::FromRow)]
struct PackInfoRow {
    version_id: i32,
    package_id: String,
    pack_type: String,
    version: String,
    language: String,
    name: String,
    description: Option<String>,
    size_bytes: i64,
    sha256: String,
    file_path: String,
}

impl From<PackInfoRow> for PackInfo {
    fn from(row: PackInfoRow) -> Self {
        Self {
            version_id: row.version_id,
            package_id: row.package_id,
            pack_type: row.pack_type,
            version: row.version,
            language: row.language,
            name: row.name,
            description: row.description,
            size_bytes: row.size_bytes,
            sha256: row.sha256,
            file_path: row.file_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    fn unreachable_pool() -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/iqrah")
            .expect("lazy pool should be created")
    }

    #[tokio::test]
    async fn repository_methods_return_query_errors_without_database() {
        let repo = PackRepository::new(unreachable_pool());

        assert!(matches!(
            repo.list_available(None, None).await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.get_pack("pack").await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.list_active_pack_versions().await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.list_all_packs().await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.get_active_version_id("pack").await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.register_pack("pack", "type", "en", "name", None).await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.add_version("pack", "1.0.0", "file", 10, "sha", None)
                .await,
            Err(StorageError::Query(_))
        ));
        assert!(matches!(
            repo.publish_pack("pack").await,
            Err(StorageError::Query(_))
        ));
    }
}
