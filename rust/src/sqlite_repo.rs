use crate::repository::{
    DebugStats, DueItem, KnowledgeGraphRepository, MemoryState, NodeData, ReviewGrade,
};
use anyhow::Result;
use async_trait::async_trait;
use flutter_rust_bridge::frb;
use fsrs::FSRS;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{config::DbConfig, params};
use std::path::PathBuf;
use tokio::task;

fn calculate_energy_delta(grade: ReviewGrade, current_energy: f64) -> f64 {
    let base_delta = match grade {
        ReviewGrade::Again => -0.1,
        ReviewGrade::Hard => 0.02,
        ReviewGrade::Good => 0.05,
        ReviewGrade::Easy => 0.08,
    };
    // Diminishing returns as energy approaches 1.0
    base_delta * (1.0 - current_energy)
}

#[frb(ignore)]
pub struct SqliteRepository {
    pool: Pool<SqliteConnectionManager>,
}

#[frb(ignore)]
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
}

#[frb(ignore)]
#[async_trait]
impl KnowledgeGraphRepository for SqliteRepository {
    async fn seed(&self) -> Result<()> {
        let pool = self.pool.clone();
        task::spawn_blocking(move || {
            let conn = pool.get()?;
            crate::database::seed_database(&conn)
        })
        .await?
    }

    async fn get_due_items(&self, user_id: &str, limit: u32) -> Result<Vec<NodeData>> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
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
        })
        .await?
    }

    async fn get_node_data(&self, node_id: &str) -> Result<NodeData> {
        let pool = self.pool.clone();
        let node_id = node_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
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
        })
        .await?
    }

    async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<MemoryState> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();
        let node_id = node_id.to_string();

        task::spawn_blocking(move || {
        let conn = pool.get()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Get current state - no JSON parsing needed
        let (stability, difficulty, energy, review_count, last_reviewed): (f64, f64, f64, i32, i64) = conn.query_row(
            "SELECT stability, difficulty, energy, review_count, last_reviewed FROM user_memory_states WHERE user_id = ? AND node_id = ?",
            params![user_id, node_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        )?;

        // Calculate elapsed days using chrono
        let elapsed_days = if last_reviewed > 0 {
            let last_review_time = chrono::DateTime::from_timestamp_millis(last_reviewed)
                .unwrap_or_else(chrono::Utc::now);
            let now = chrono::Utc::now();
            (now - last_review_time).num_days().max(0) as u32
        } else {
            0
        };

        // Create FSRS MemoryState from database values
        let memory_state = if review_count > 0 {
            Some(fsrs::MemoryState {
                stability: stability as f32,
                difficulty: difficulty as f32,
            })
        } else {
            None // New card
        };

        // FSRS calculation
        let fsrs = FSRS::new(Some(&[])).map_err(|e| anyhow::anyhow!("FSRS init failed: {}", e))?;
        let optimal_retention = 0.8;

        let next_states = fsrs.next_states(memory_state, optimal_retention, elapsed_days)
            .map_err(|e| anyhow::anyhow!("FSRS calculation failed: {}", e))?;

        // Select the appropriate state based on grade
        let selected_state = match grade {
            ReviewGrade::Again => next_states.again,
            ReviewGrade::Hard => next_states.hard,
            ReviewGrade::Good => next_states.good,
            ReviewGrade::Easy => next_states.easy,
        };

        // Calculate due date from interval
        let due_at_ms = now_ms + (selected_state.interval as i64 * 24 * 60 * 60 * 1000);

        // Energy calculation
        let energy_delta = calculate_energy_delta(grade, energy);
        let new_energy = (energy + energy_delta).clamp(0.0, 1.0);

        // Update database - store stability/difficulty directly
        conn.execute(
            "UPDATE user_memory_states
             SET stability = ?, difficulty = ?, energy = ?, last_reviewed = ?, due_at = ?, review_count = review_count + 1
             WHERE user_id = ? AND node_id = ?",
            params![
                selected_state.memory.stability as f64,
                selected_state.memory.difficulty as f64,
                new_energy,
                now_ms,
                due_at_ms,
                user_id,
                node_id
            ]
        )?;

        // Simple propagation
        if energy_delta > 0.05 {
            let arabic_word: String = conn.query_row(
                "SELECT value FROM node_metadata WHERE node_id = ? AND key = 'arabic'",
                params![node_id], |row| row.get(0)
            ).unwrap_or_default();

            if !arabic_word.is_empty() {
                conn.execute(
                    "UPDATE user_memory_states SET energy = MIN(1.0, energy + ?)
                     WHERE user_id = ? AND node_id IN (
                       SELECT node_id FROM node_metadata
                       WHERE key='arabic' AND value=? AND node_id != ?
                     ) AND energy < 0.8",
                    params![energy_delta * 0.5, user_id, arabic_word, node_id]
                )?;
            }
        }

        Ok(MemoryState {
            stability: selected_state.memory.stability as f64,
            difficulty: selected_state.memory.difficulty as f64,
            energy: new_energy,
            due_at: due_at_ms,
            review_count: review_count + 1,
            last_reviewed: now_ms,
        })
    }).await?
    }

    async fn get_debug_stats(&self, user_id: &str) -> Result<DebugStats> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now_ms = chrono::Utc::now().timestamp_millis();

            let due_today: u32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM user_memory_states WHERE user_id = ? AND due_at <= ?",
                    params![user_id, now_ms],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0) as u32;

            let avg_energy: f64 = conn.query_row(
            "SELECT AVG(energy) FROM user_memory_states WHERE user_id = ? AND review_count > 0",
            params![user_id], |row| row.get(0)
        ).unwrap_or(0.0);

            let total_reviewed: u32 = conn.query_row(
            "SELECT COUNT(*) FROM user_memory_states WHERE user_id = ? AND review_count > 0",
            params![user_id], |row| row.get::<_, i32>(0)
        ).unwrap_or(0) as u32;

            // Get next 5 due items
            let mut stmt = conn.prepare(
                "SELECT ums.node_id, ums.stability, ums.difficulty, ums.energy, ums.last_reviewed, ums.due_at, ums.review_count,
                        MAX(CASE WHEN nm.key = 'arabic' THEN nm.value END) AS arabic
                 FROM user_memory_states ums
                 JOIN nodes n ON ums.node_id = n.id
                 LEFT JOIN node_metadata nm ON ums.node_id = nm.node_id
                 WHERE ums.user_id = ?
                 GROUP BY ums.node_id
                 ORDER BY ums.due_at LIMIT 5",
            )?;

            let next_due_items: Vec<DueItem> = stmt
                .query_map(params![user_id], |row| {
                    Ok(DueItem {
                        node_id: row.get(0)?,
                        state: MemoryState {
                            stability: row.get(1)?,
                            difficulty: row.get(2)?,
                            energy: row.get(3)?,
                            last_reviewed: row.get(4)?,
                            due_at: row.get(5)?,
                            review_count: row.get(6)?,
                        },
                        arabic: row.get(7)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(DebugStats {
                due_today,
                total_reviewed,
                avg_energy,
                next_due_items,
            })
        })
        .await?
    }
}
