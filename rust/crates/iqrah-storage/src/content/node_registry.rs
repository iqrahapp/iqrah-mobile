use sqlx::{query_as, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A caching layer to map string unique keys (ukeys) to integer IDs.
/// This is a critical component for the "Internal Ints, External Strings" architecture.
#[derive(Debug, Clone)]
pub struct NodeRegistry {
    cache: Arc<RwLock<HashMap<String, i64>>>,
    pool: SqlitePool,
}

impl NodeRegistry {
    /// Creates a new `NodeRegistry`.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            pool,
        }
    }

    /// Pre-populates the cache with all nodes from the database.
    pub async fn load_all(&self) -> anyhow::Result<()> {
        let rows = query_as::<_, (i64, String)>("SELECT id, ukey FROM nodes")
            .fetch_all(&self.pool)
            .await?;

        let mut cache = self.cache.write().await;
        cache.clear();
        for (id, ukey) in rows {
            cache.insert(ukey, id);
        }

        Ok(())
    }

    /// Resolves a string ukey to an integer ID.
    ///
    /// This method first checks the in-memory cache. If the ukey is not found,
    /// it queries the database and, if successful, caches the result for future lookups.
    pub async fn get_id(&self, ukey: &str) -> anyhow::Result<Option<i64>> {
        // Check cache first
        let cache = self.cache.read().await;
        if let Some(id) = cache.get(ukey) {
            return Ok(Some(*id));
        }
        drop(cache); // Release read lock

        // If not in cache, query the database
        let row = query_as::<_, (i64,)>("SELECT id FROM nodes WHERE ukey = ?")
            .bind(ukey)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((id,)) = row {
            // Found in DB, so cache it
            let mut cache = self.cache.write().await;
            cache.insert(ukey.to_string(), id);
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}
