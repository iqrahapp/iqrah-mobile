use sqlx::SqlitePool;
use anyhow::Result;
use std::path::Path;

/// Check if old database exists
pub fn old_db_exists(old_db_path: &str) -> bool {
    Path::new(old_db_path).exists()
}

/// Migrate data from old single database to new two-database architecture
pub async fn migrate_from_old_db(
    _old_db_path: &str,
    _content_pool: &SqlitePool,
    _user_pool: &SqlitePool,
) -> Result<()> {
    // Implementation will be added in Step 6
    Ok(())
}

/// Mark migration as complete by creating a marker file
pub fn mark_migration_complete(marker_path: &str) -> Result<()> {
    use std::fs;
    fs::write(marker_path, "migrated")?;
    Ok(())
}

/// Check if migration has been completed
pub fn is_migration_complete(marker_path: &str) -> bool {
    Path::new(marker_path).exists()
}
