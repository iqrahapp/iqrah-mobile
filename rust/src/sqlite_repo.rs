use crate::repository::{KnowledgeGraphRepository, MemoryState, NodeData, ReviewGrade};
use anyhow::Result;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{config::DbConfig, params};
use std::path::PathBuf;

pub struct SqliteRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteRepository {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let manager = match path {
            Some(p) => SqliteConnectionManager::file(p),
            None => SqliteConnectionManager::memory(),
        };
        let pool = Pool::builder().max_size(8).build(manager)?;

        // One-time setup on a connection
        {
            let conn = pool.get()?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true)?;

            // Run schema setup
            crate::database::create_schema(&conn)?;
        }

        Ok(Self { pool })
    }

    pub fn seed(&self) -> Result<()> {
        let conn = self.pool.get()?;
        crate::database::seed_database(&conn)
    }

    fn conn(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }
}

impl KnowledgeGraphRepository for SqliteRepository {
    fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT n.id,
                    MAX(CASE WHEN nm.key = 'arabic' THEN nm.value END) AS arabic,
                    MAX(CASE WHEN nm.key = 'translation' THEN nm.value END) AS translation
             FROM nodes n
             JOIN node_metadata nm ON n.id = nm.node_id
             JOIN user_memory_states ums ON n.id = ums.node_id
             WHERE ums.user_id = ? AND ums.due_at <= ?
             GROUP BY n.id
             ORDER BY ums.last_reviewed ASC
             LIMIT ?",
        )?;

        let now_ms = chrono::Utc::now().timestamp_millis();
        let items = stmt
            .query_map(params![user_id, now_ms, limit], |row| {
                Ok(NodeData {
                    id: row.get("id")?,
                    arabic: row.get("arabic")?,
                    translation: row.get("translation")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(items)
    }

    fn get_node_data(&self, node_id: &str) -> Result<NodeData> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT n.id,
                    MAX(CASE WHEN nm.key = 'arabic' THEN nm.value END) AS arabic,
                    MAX(CASE WHEN nm.key = 'translation' THEN nm.value END) AS translation
             FROM nodes n
             JOIN node_metadata nm ON n.id = nm.node_id
             WHERE n.id = ?
             GROUP BY n.id",
        )?;

        let node = stmt.query_row(params![node_id], |row| {
            Ok(NodeData {
                id: row.get("id")?,
                arabic: row.get("arabic")?,
                translation: row.get("translation")?,
            })
        })?;

        Ok(node)
    }

    fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        let conn = self.conn()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Simple MVP scheduling
        let (stability, difficulty, days_ahead) = match grade {
            ReviewGrade::Again => (0.5, 8.0, 1),
            ReviewGrade::Hard => (1.0, 6.0, 3),
            ReviewGrade::Good => (2.0, 4.0, 7),
            ReviewGrade::Easy => (4.0, 2.0, 14),
        };

        let due_at = now_ms + (days_ahead * 24 * 60 * 60 * 1000);

        conn.execute(
            "UPDATE user_memory_states
             SET stability = ?, difficulty = ?, last_reviewed = ?, due_at = ?, review_count = review_count + 1
             WHERE user_id = ? AND node_id = ?",
            params![stability, difficulty, now_ms, due_at, user_id, node_id],
        )?;

        let review_count: i32 = conn.query_row(
            "SELECT review_count FROM user_memory_states WHERE user_id = ? AND node_id = ?",
            params![user_id, node_id],
            |row| row.get(0),
        )?;

        Ok(MemoryState {
            stability,
            difficulty,
            due_at,
            review_count,
            last_reviewed: now_ms,
        })
    }
}
