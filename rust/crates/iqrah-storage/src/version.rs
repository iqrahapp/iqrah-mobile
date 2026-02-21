use crate::error::Result;
use sqlx::SqlitePool;

/// Get the current schema version from the database
pub async fn get_schema_version(pool: &SqlitePool) -> Result<String> {
    let row =
        sqlx::query_scalar!("SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1")
            .fetch_one(pool)
            .await?;

    Ok(row)
}

/// Check if database schema version is compatible with app version
pub fn is_compatible(db_version: &str, app_version: &str) -> bool {
    let db_parts = parse_version(db_version);
    let app_parts = parse_version(app_version);

    // Major version must match (breaking changes)
    if db_parts.0 != app_parts.0 {
        return false;
    }

    // Minor version: DB can be <= app version (backwards compatible)
    if db_parts.1 > app_parts.1 {
        return false; // DB is newer than app
    }

    // Patch version doesn't affect compatibility
    true
}

fn parse_version(version: &str) -> (u32, u32, u32) {
    let parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();

    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        // Same version
        assert!(is_compatible("2.0.0", "2.0.0"));

        // App newer (minor): Compatible
        assert!(is_compatible("2.0.0", "2.1.0"));

        // DB newer (minor): Incompatible
        assert!(!is_compatible("2.1.0", "2.0.0"));

        // Different major: Incompatible
        assert!(!is_compatible("1.0.0", "2.0.0"));
        assert!(!is_compatible("2.0.0", "1.0.0"));

        // Patch differences: Always compatible
        assert!(is_compatible("2.0.0", "2.0.5"));
        assert!(is_compatible("2.0.5", "2.0.0"));
    }
}
