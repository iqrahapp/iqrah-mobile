use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// NodeRegistry manages the mapping between string-based unique keys (ukeys)
/// and integer-based node IDs for high-performance lookups.
///
/// The registry provides:
/// - Bidirectional mapping: ukey <-> i64
/// - In-memory caching for fast repeated lookups
/// - Thread-safe concurrent access
#[derive(Clone)]
pub struct NodeRegistry {
    /// In-memory cache: ukey -> i64
    ukey_cache: Arc<RwLock<HashMap<String, i64>>>,
    /// In-memory cache: i64 -> ukey
    id_cache: Arc<RwLock<HashMap<i64, String>>>,
    /// Database pool for fallback queries
    pool: SqlitePool,
}

impl NodeRegistry {
    /// Create a new NodeRegistry
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            ukey_cache: Arc::new(RwLock::new(HashMap::new())),
            id_cache: Arc::new(RwLock::new(HashMap::new())),
            pool,
        }
    }

    /// Get the integer ID for a given unique key (ukey).
    /// Returns None if the node doesn't exist in the registry.
    ///
    /// This method checks the cache first, then falls back to database query.
    pub async fn get_id(&self, ukey: &str) -> anyhow::Result<Option<i64>> {
        // Check cache first
        {
            let cache = self.ukey_cache.read().await;
            if let Some(&id) = cache.get(ukey) {
                return Ok(Some(id));
            }
        }

        // Query database
        let result = sqlx::query!(
            "SELECT
                id AS \"id!: i64\",
                ukey AS \"ukey!: String\"
             FROM nodes
             WHERE ukey = ?",
            ukey
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            let id = row.id;
            let fetched_ukey = row.ukey;
            // Update both caches
            let mut ukey_cache = self.ukey_cache.write().await;
            let mut id_cache = self.id_cache.write().await;
            ukey_cache.insert(fetched_ukey.clone(), id);
            id_cache.insert(id, fetched_ukey);
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Get the unique key (ukey) for a given integer ID.
    /// Returns None if the node doesn't exist in the registry.
    pub async fn get_ukey(&self, id: i64) -> anyhow::Result<Option<String>> {
        // Check cache first
        {
            let cache = self.id_cache.read().await;
            if let Some(ukey) = cache.get(&id) {
                return Ok(Some(ukey.clone()));
            }
        }

        // Query database
        let result = sqlx::query!(
            "SELECT
                id AS \"id!: i64\",
                ukey AS \"ukey!: String\"
             FROM nodes
             WHERE id = ?",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            let fetched_id = row.id;
            let ukey = row.ukey;
            // Update both caches
            let mut ukey_cache = self.ukey_cache.write().await;
            let mut id_cache = self.id_cache.write().await;
            ukey_cache.insert(ukey.clone(), fetched_id);
            id_cache.insert(fetched_id, ukey.clone());
            Ok(Some(ukey))
        } else {
            Ok(None)
        }
    }

    /// Check if a node exists in the registry by ukey
    pub async fn exists_by_ukey(&self, ukey: &str) -> anyhow::Result<bool> {
        Ok(self.get_id(ukey).await?.is_some())
    }

    /// Check if a node exists in the registry by ID
    pub async fn exists_by_id(&self, id: i64) -> anyhow::Result<bool> {
        Ok(self.get_ukey(id).await?.is_some())
    }

    /// Register a new node in the registry (inserts into database and cache)
    pub async fn register(&self, id: i64, ukey: String, node_type: i32) -> anyhow::Result<()> {
        let ukey_ref = ukey.as_str();
        sqlx::query!(
            "INSERT OR IGNORE INTO nodes (id, ukey, node_type) VALUES (?, ?, ?)",
            id,
            ukey_ref,
            node_type
        )
        .execute(&self.pool)
        .await?;

        // Update caches
        let mut ukey_cache = self.ukey_cache.write().await;
        let mut id_cache = self.id_cache.write().await;
        ukey_cache.insert(ukey.clone(), id);
        id_cache.insert(id, ukey);

        Ok(())
    }

    /// Clear the in-memory caches (useful for testing or memory management)
    pub async fn clear_cache(&self) {
        let mut ukey_cache = self.ukey_cache.write().await;
        let mut id_cache = self.id_cache.write().await;
        ukey_cache.clear();
        id_cache.clear();
    }

    /// Get cache statistics (for debugging/monitoring)
    pub async fn cache_stats(&self) -> (usize, usize) {
        let ukey_cache = self.ukey_cache.read().await;
        let id_cache = self.id_cache.read().await;
        (ukey_cache.len(), id_cache.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iqrah_core::domain::node_id as nid;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create nodes table
        sqlx::query(
            "CREATE TABLE nodes (
                id INTEGER PRIMARY KEY,
                ukey TEXT NOT NULL UNIQUE,
                node_type INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_register_and_retrieve() {
        let pool = setup_test_db().await;
        let registry = NodeRegistry::new(pool);

        let id = nid::encode_verse(1, 1);
        let ukey = nid::verse(1, 1);

        // Register
        registry.register(id, ukey.clone(), 1).await.unwrap();

        // Retrieve by ukey
        let retrieved_id = registry.get_id(&ukey).await.unwrap();
        assert_eq!(retrieved_id, Some(id));

        // Retrieve by id
        let retrieved_ukey = registry.get_ukey(id).await.unwrap();
        assert_eq!(retrieved_ukey, Some(ukey));
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let pool = setup_test_db().await;
        let registry = NodeRegistry::new(pool);

        let id = nid::encode_verse(1, 1);
        let ukey = nid::verse(1, 1);

        registry.register(id, ukey.clone(), 1).await.unwrap();

        // First lookup populates cache
        let _ = registry.get_id(&ukey).await.unwrap();

        // Check cache stats
        let (ukey_cache_size, id_cache_size) = registry.cache_stats().await;
        assert_eq!(ukey_cache_size, 1);
        assert_eq!(id_cache_size, 1);

        // Second lookup should hit cache (we can't directly test this,
        // but cache_stats confirms it's populated)
        let retrieved_id = registry.get_id(&ukey).await.unwrap();
        assert_eq!(retrieved_id, Some(id));
    }

    #[tokio::test]
    async fn test_nonexistent_node() {
        let pool = setup_test_db().await;
        let registry = NodeRegistry::new(pool);

        let result = registry.get_id("VERSE:999:999").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_exists() {
        let pool = setup_test_db().await;
        let registry = NodeRegistry::new(pool);

        let id = nid::encode_verse(1, 1);
        let ukey = nid::verse(1, 1);

        registry.register(id, ukey.clone(), 1).await.unwrap();

        assert!(registry.exists_by_ukey(&ukey).await.unwrap());
        assert!(registry.exists_by_id(id).await.unwrap());
        assert!(!registry.exists_by_ukey("VERSE:999:999").await.unwrap());
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let pool = setup_test_db().await;
        let registry = NodeRegistry::new(pool);

        let id = nid::encode_verse(1, 1);
        let ukey = nid::verse(1, 1);

        registry.register(id, ukey.clone(), 1).await.unwrap();

        // Populate cache
        let _ = registry.get_id(&ukey).await.unwrap();

        let (before_clear, _) = registry.cache_stats().await;
        assert_eq!(before_clear, 1);

        // Clear cache
        registry.clear_cache().await;

        let (after_clear, _) = registry.cache_stats().await;
        assert_eq!(after_clear, 0);

        // Data should still be in database
        let retrieved_id = registry.get_id(&ukey).await.unwrap();
        assert_eq!(retrieved_id, Some(id));
    }
}
