use crate::{
    cbor_import::{ImportedEdge, ImportedNode, NodeType},
    propagation::{DistributionParams, DistributionType, EdgeForPropagation, EdgeType},
    repository::{
        DebugStats, DueItem, ItemPreview, KnowledgeGraphRepository, MemoryState, NodeData,
        ReviewGrade, ScoreBreakdown, ScoreWeights,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use flutter_rust_bridge::frb;
use fsrs::FSRS;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{config::DbConfig, params, OptionalExtension};
use std::{collections::HashMap, path::PathBuf};
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

    /*
    ✅ Good News: You've Already Implemented the Core Idea!

    Before exploring more advanced techniques, I want to point out that your existing calculate_energy_delta function is smarter than you might think. Look at this part:

    Rust
    // This is your current, elegant solutionfn calculate_energy_delta(grade: ReviewGrade, current_energy: f64) -> f64 {

        let base_delta = /* ... */;

        // This part is key!

        base_delta * (1.0 - current_energy.max(0.0)) // (using max(0.0) to handle the [-1, 1] range)

    }

    The * (1.0 - current_energy) term already implements an adaptive delta based on diminishing returns.

    When energy is low (e.g., 0.1), the multiplier is large (* 0.9), resulting in a big step.

    When energy is high (e.g., 0.9), the multiplier is small (* 0.1), resulting in a small, fine-tuning step.

    This is an excellent, simple, and effective MVP for an adaptive system.

    🚀 How to Make It Even Smarter (Ideas for Future Sprints)

    While your current model is great, we can get even closer to an Adam-like optimizer by incorporating more context. Here are two advanced approaches for a future sprint.

    1. The "Momentum" Model

    This approach adds a "velocity" to learning. If a user consistently answers correctly, the energy gains could accelerate.

    How it works: Add a learning_momentum column (e.g., from 0.5 to 1.5) to the user_memory_states table.

    On a good review: Slightly increase the momentum (e.g., momentum = (momentum + 0.1).min(1.5)).

    On a bad review: Reset the momentum back to a lower value (e.g., 1.0 or 0.8).

    The new delta calculation: final_delta = base_delta * (1.0 - current_energy) * learning_momentum.

    This allows the system to differentiate between a lucky guess and consistent knowledge.

    2. The FSRS-Integrated Model (The Principled Approach)

    This is the most elegant solution because it uses data you already have. The FSRS engine calculates Stability (S), which is a fantastic proxy for long-term memory strength.

    The Logic:

    When Stability is low, the concept is new or forgotten. We need a large energy delta.

    When Stability is high, the concept is well-mastered. We need a small energy delta to avoid overshooting.

    How it works: We can use a function that decays as stability increases. The tanh function is perfect for this as it smoothly maps the unbounded stability value.

    Rust



    // In process_review, you already have the `stability` value from FSRSlet stability = selected_state.memory.stability;// Create a factor that decreases as stability increases// The number `30.0` is a tuning parameter: a larger value means the factor decays slower.let stability_factor = 1.0 - (stability / 30.0).tanh(); let final_delta = base_delta * stability_factor;

    This method is powerful because it directly ties your two memory systems together. The FSRS scheduling engine's assessment of memory strength now directly informs the energy propagation system.

    Verdict: Your concern is valid and shows a deep understanding of the problem. For now, be confident that your current "diminishing returns" model is a solid and effective MVP. For a future sprint, the FSRS-Integrated Model is the ideal, principled path forward.
    */
}

#[frb(ignore)]
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
}

// #[frb(ignore)]
#[async_trait]
impl KnowledgeGraphRepository for SqliteRepository {
    async fn get_due_items(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
    ) -> Result<Vec<NodeData>> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;

            // Build the WHERE clause dynamically based on surah_filter
            let (where_clause, _) = match surah_filter {
                Some(_) => {
                    // For word_instance: check parent_node points to verse that starts with "VERSE:{chapter_num}:"
                    // For verse: check the ID starts with "VERSE:{chapter_num}:"
                    (
                        "AND ((n.node_type = 'word_instance' AND EXISTS (
                            SELECT 1 FROM node_metadata nm_parent
                            WHERE nm_parent.node_id = n.id
                            AND nm_parent.key = 'parent_node'
                            AND nm_parent.value LIKE 'VERSE:' || ?4 || ':%'
                        )) OR (n.node_type = 'verse' AND n.id LIKE 'VERSE:' || ?4 || ':%'))",
                        4
                    )
                }
                None => ("", 3)
            };

            let query = format!(
                "SELECT n.id, n.node_type, nm.key, nm.value
                 FROM nodes n
                 JOIN node_metadata nm ON n.id = nm.node_id
                 JOIN user_memory_states ums ON n.id = ums.node_id
                 WHERE ums.user_id = ?1 AND ums.due_at <= ?2
                   AND n.node_type IN ('word_instance', 'verse')
                   {}
                 ORDER BY (
                     1.0 * MAX(0, (?2 - ums.due_at) / (24.0 * 60.0 * 60.0 * 1000.0)) +
                     2.0 * MAX(0, 1.0 - ums.energy) +
                     1.5 * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata nm2
                                     WHERE nm2.node_id = n.id AND nm2.key = 'foundational_score'), 0)
                 ) DESC, ums.last_reviewed ASC
                 LIMIT ?3",
                where_clause
            );

            let mut stmt = conn.prepare(&query)?;

            let now_ms = chrono::Utc::now().timestamp_millis();

            // Create a helper function to avoid closure type mismatch
            fn map_row(row: &rusqlite::Row) -> rusqlite::Result<(String, NodeType, String, String)> {
                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, NodeType>("node_type")?,
                    row.get::<_, String>("key")?,
                    row.get::<_, String>("value")?,
                ))
            }

            let rows = match surah_filter {
                Some(chapter_num) => {
                    stmt.query_map(params![user_id, now_ms, limit, chapter_num], map_row)?
                }
                None => {
                    stmt.query_map(params![user_id, now_ms, limit], map_row)?
                }
            };

            // Group metadata by node_id
            let mut nodes_map: HashMap<String, NodeData> = HashMap::new();

            for row in rows {
                let (node_id, node_type, key, value) = row?;

                nodes_map
                    .entry(node_id.clone())
                    .or_insert_with(|| NodeData {
                        id: node_id.clone(),
                        node_type,
                        metadata: HashMap::new(),
                    })
                    .metadata
                    .insert(key, value);
            }

            // Convert to NodeData and take only the limit we need
            Ok(nodes_map
                .into_values()
                // .take(limit as usize) // Already limited by SQL query
                .collect::<Vec<_>>())
        })
        .await?
    }

    async fn process_review(
        &self,
        user_id: &str,
        node_id: &str,
        grade: ReviewGrade,
    ) -> Result<(MemoryState, f64)> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();
        let node_id = node_id.to_string();

        task::spawn_blocking(move || {
        let conn = pool.get()?;
        let now_ms = chrono::Utc::now().timestamp_millis();

        // Get current state
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
        let new_energy = (energy + energy_delta).clamp(-1.0, 1.0);

        // Update database - store stability/difficulty directly
        conn.execute(
            "UPDATE user_memory_states
            SET stability = ?, difficulty = ?, energy = ?, last_reviewed = ?, due_at = ?,
                review_count = review_count + 1,
                priority_score = (
                    1.0 * MAX(0, (? - ?) / (24.0 * 60.0 * 60.0 * 1000.0)) +
                    2.0 * MAX(0, 1.0 - ?) +
                    1.5 * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata
                                    WHERE node_id = ? AND key = 'foundational_score'), 0)
                )
            WHERE user_id = ? AND node_id = ?",
            params![
                selected_state.memory.stability as f64,  // 1
                selected_state.memory.difficulty as f64, // 2
                new_energy,                              // 3
                now_ms,                                  // 4
                due_at_ms,                              // 5
                now_ms,                                 // 6 - for (? - ?) overdue calculation
                due_at_ms,                              // 7 - for (? - ?) overdue calculation
                new_energy,                             // 8 - for (1.0 - ?) mastery gap
                node_id,                                // 9 - for foundational_score lookup
                user_id,                                // 10
                node_id                                 // 11
            ]
        )?;

        Ok((MemoryState {
            stability: selected_state.memory.stability as f64,
            difficulty: selected_state.memory.difficulty as f64,
            energy: new_energy,
            due_at: due_at_ms,
            review_count: review_count + 1,
            last_reviewed: now_ms,
        }, new_energy))
    }).await?
    }

    async fn get_knowledge_edges(&self, source_node_id: &str) -> Result<Vec<EdgeForPropagation>> {
        let pool = self.pool.clone();
        let source_node_id = source_node_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get().map_err(|e| anyhow::anyhow!(e))?;
            let mut stmt = conn.prepare(
                "SELECT target_id, distribution_type, param1, param2
                 FROM edges
                 WHERE source_id = ? AND edge_type = ?",
            )?;

            let edges = stmt
                .query_map(params![source_node_id, EdgeType::Knowledge as i32], |row| {
                    let dist_type_int: i32 = row.get(1)?; // Read as integer
                    let p1: f32 = row.get(2)?;
                    let p2: f32 = row.get(3)?;

                    // Convert integer to enum, falling back to Constant on error
                    let distribution = match DistributionType::try_from(dist_type_int) {
                        Ok(DistributionType::Normal) => DistributionParams::Normal {
                            mean: p1,
                            std_dev: p2,
                        },
                        Ok(DistributionType::Beta) => DistributionParams::Beta {
                            alpha: p1,
                            beta: p2,
                        },
                        _ => DistributionParams::Constant { weight: p1 }, // Default/fallback
                    };

                    Ok(EdgeForPropagation {
                        target_node_id: row.get(0)?,
                        distribution,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(edges)
        })
        .await?
    }

    async fn get_node_energy(&self, user_id: &str, node_id: &str) -> Result<Option<f64>> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();
        let node_id = node_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get().map_err(|e| anyhow::anyhow!(e))?;
            let energy = conn
                .query_row(
                    "SELECT energy FROM user_memory_states WHERE user_id = ? AND node_id = ?",
                    params![user_id, node_id],
                    |row| row.get(0),
                )
                .optional()?;
            Ok(energy)
        })
        .await?
    }

    async fn update_node_energies(&self, user_id: &str, updates: &[(String, f64)]) -> Result<()> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();
        let updates = updates.to_vec();

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;
            let now_ms = chrono::Utc::now().timestamp_millis();

            for (node_id, new_energy) in updates {
                tx.execute(
                    "UPDATE user_memory_states
                 SET energy = ?1,
                     priority_score = (
                         1.0 * MAX(0, (?2 - due_at) / (24.0 * 60.0 * 60.0 * 1000.0)) +
                         2.0 * MAX(0, 1.0 - ?1) +
                         1.5 * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata nm
                                        WHERE nm.node_id = ?3 AND nm.key = 'foundational_score'), 0)
                     )
                 WHERE user_id = ?4 AND node_id = ?3",
                    params![new_energy, now_ms, node_id, user_id],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
        .await?
    }

    async fn refresh_all_priority_scores(&self, user_id: &str) -> Result<()> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now_ms = chrono::Utc::now().timestamp_millis();

            conn.execute(
                "
            UPDATE user_memory_states
            SET priority_score = (
                1.0 * MAX(0, (?1 - due_at) / (24.0 * 60.0 * 60.0 * 1000.0)) +
                2.0 * MAX(0, 1.0 - energy) +
                1.5 * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata nm
                               WHERE nm.node_id = user_memory_states.node_id
                               AND nm.key = 'foundational_score'), 0)
            )
            WHERE user_id = ?",
                params![now_ms, user_id],
            )?;
            Ok(())
        })
        .await?
    }

    async fn sync_user_nodes(&self, user_id: &str) -> Result<()> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || -> anyhow::Result<()> {
            let conn = pool.get()?;

            let now_ms = chrono::Utc::now().timestamp_millis();

            conn.execute(
                "INSERT OR IGNORE INTO user_memory_states (user_id, node_id, due_at)
             SELECT ?1, id, ?2 FROM nodes",
                params![user_id, now_ms],
            )?;

            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn reset_user_progress(&self, user_id: &str) -> Result<()> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || -> anyhow::Result<()> {
            let conn = pool.get()?;
            let now_ms = chrono::Utc::now().timestamp_millis();

            // This single command efficiently replaces all existing records or inserts
            // new ones for the user, resetting their progress for every node.
            conn.execute(
                "INSERT OR REPLACE INTO user_memory_states (user_id, node_id, due_at)
             SELECT ?1, id, ?2 FROM nodes",
                params![user_id, now_ms],
            )?;

            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn insert_nodes_batch(&self, nodes: &[ImportedNode]) -> Result<()> {
        let pool = self.pool.clone();
        // We need to own the data to move it into the thread
        let nodes_data = nodes.to_vec();

        Ok(task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut conn = pool.get()?;
            // Use a transaction for a massive speed boost on bulk inserts
            let tx = conn.transaction()?;

            {
                // Prepare statements once before the loop
                let mut insert_node = tx.prepare_cached(
                    "INSERT OR REPLACE INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)",
                )?;
                let mut insert_meta = tx.prepare_cached(
                    "INSERT OR REPLACE INTO node_metadata (node_id, key, value) VALUES (?, ?, ?)",
                )?;
                let now_ms = chrono::Utc::now().timestamp_millis();

                for node in &nodes_data {
                    // 1. Insert into the main `nodes` table
                    insert_node.execute(params![node.id, node.attributes.node_type, now_ms])?;

                    // 2. Insert all metadata into the `node_metadata` table
                    for (key, value) in &node.attributes.metadata {
                        insert_meta.execute(params![node.id, key, value])?;
                    }
                }
            } // Statements are dropped here before the transaction is committed

            tx.commit()?;
            Ok(())
        })
        .await??)
    }

    async fn insert_edges_batch(&self, edges: &[ImportedEdge]) -> Result<()> {
        let pool = self.pool.clone();
        let edges_data = edges.to_vec();

        Ok(task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut conn = pool.get()?;
        let tx = conn.transaction()?;

            {
                let mut insert_edge = tx.prepare_cached(
                    "INSERT OR REPLACE INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2) VALUES (?, ?, ?, ?, ?, ?)",
                )?;

                for edge in &edges_data {
                    let (dist_type, p1, p2) = match edge.distribution {
                        DistributionParams::Normal { mean, std_dev } => (DistributionType::Normal, mean, std_dev),
                        DistributionParams::Beta { alpha, beta } => (DistributionType::Beta, alpha, beta),
                        DistributionParams::Constant { weight } => (DistributionType::Constant, weight, 0.0),
                    };

                    insert_edge.execute(params![edge.source_id, edge.target_id, edge.edge_type as i32, dist_type as i32, p1, p2])?;
                }
            }

            tx.commit()?;
            Ok(())
        })
        .await??)
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


            let total_nodes_count  : usize = conn.query_one("SELECT COUNT(*) FROM nodes", [], |row| row.get::<_, usize>(0)).unwrap_or(0);
            let total_edges_count  : usize = conn.query_one("SELECT COUNT(*) FROM edges", [], |row| row.get::<_, usize>(0)).unwrap_or(0);


            Ok(DebugStats {
                due_today,
                total_reviewed,
                avg_energy,
                next_due_items,
                total_nodes_count,
                total_edges_count,
            })
        })
        .await?
    }

    async fn get_session_preview(
        &self,
        user_id: &str,
        limit: u32,
        surah_filter: Option<i32>,
    ) -> Result<Vec<ItemPreview>> {
        let pool = self.pool.clone();
        let user_id = user_id.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let now_ms = chrono::Utc::now().timestamp_millis();

            // Build the WHERE clause dynamically based on surah_filter
            let where_clause = match surah_filter {
                Some(_) => {
                    "AND ((n.node_type = 'word_instance' AND EXISTS (
                        SELECT 1 FROM node_metadata nm_parent
                        WHERE nm_parent.node_id = n.id
                        AND nm_parent.key = 'parent_node'
                        AND nm_parent.value LIKE 'VERSE:' || ?4 || ':%'
                    )) OR (n.node_type = 'verse' AND n.id LIKE 'VERSE:' || ?4 || ':%'))"
                }
                None => ""
            };

            let query = format!(
                "
            SELECT n.id,
                   ums.energy,
                   ums.due_at,
                   (
                       1.0 * MAX(0, (?1 - ums.due_at) / (24.0 * 60.0 * 60.0 * 1000.0)) +
                       2.0 * MAX(0, 1.0 - ums.energy) +
                       1.5 * COALESCE((SELECT CAST(value AS REAL) FROM node_metadata nm2
                                      WHERE nm2.node_id = n.id AND nm2.key = 'foundational_score'), 0)
                   ) AS priority_score,
                   MAX(CASE WHEN nm.key = 'arabic' THEN nm.value END) as arabic,
                   MAX(CASE WHEN nm.key = 'translation' THEN nm.value END) as translation,
                   MAX(CASE WHEN nm.key = 'foundational_score' THEN nm.value END) as foundational_score
            FROM nodes n
            JOIN user_memory_states ums ON n.id = ums.node_id
            LEFT JOIN node_metadata nm ON n.id = nm.node_id
            WHERE ums.user_id = ?2 AND n.node_type IN ('word_instance', 'verse')
            {}
            GROUP BY n.id
            ORDER BY priority_score DESC
            LIMIT ?3
        ", where_clause);

            let mut stmt = conn.prepare(&query)?;

            let weights = ScoreWeights {
                w_due: 1.0,
                w_need: 2.0,
                w_yield: 1.5,
            };

            let previews: Vec<ItemPreview> = if let Some(chapter_num) = surah_filter {
                stmt.query_map(params![now_ms, user_id, limit, chapter_num], |row| {
                    let due_at: i64 = row.get("due_at")?;
                    let energy: f64 = row.get("energy")?;
                    let priority_score: f64 = row.get("priority_score")?;
                    let foundational_score: f64 = row
                        .get::<_, Option<String>>("foundational_score")?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);

                    // Recalculate components for transparency
                    let days_overdue =
                        ((now_ms - due_at) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)).max(0.0);
                    let mastery_gap = (1.0 - energy.max(0.0)).max(0.0);
                    let importance = foundational_score;

                    Ok(ItemPreview {
                        node_id: row.get("id")?,
                        arabic: row.get("arabic")?,
                        translation: row.get("translation")?,
                        priority_score,
                        score_breakdown: ScoreBreakdown {
                            days_overdue,
                            mastery_gap,
                            importance,
                            weights: weights.clone(),
                        },
                    })
                })?.collect::<Result<Vec<_>, _>>()?
            } else {
                stmt.query_map(params![now_ms, user_id, limit], |row| {
                    let due_at: i64 = row.get("due_at")?;
                    let energy: f64 = row.get("energy")?;
                    let priority_score: f64 = row.get("priority_score")?;
                    let foundational_score: f64 = row
                        .get::<_, Option<String>>("foundational_score")?
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);

                    // Recalculate components for transparency
                    let days_overdue =
                        ((now_ms - due_at) as f64 / (24.0 * 60.0 * 60.0 * 1000.0)).max(0.0);
                    let mastery_gap = (1.0 - energy.max(0.0)).max(0.0);
                    let importance = foundational_score;

                    Ok(ItemPreview {
                        node_id: row.get("id")?,
                        arabic: row.get("arabic")?,
                        translation: row.get("translation")?,
                        priority_score,
                        score_breakdown: ScoreBreakdown {
                            days_overdue,
                            mastery_gap,
                            importance,
                            weights: weights.clone(),
                        },
                    })
                })?.collect::<Result<Vec<_>, _>>()?
            };

            Ok(previews)
        })
        .await?
    }

    async fn get_available_surahs(&self) -> Result<Vec<(i32, String)>> {
        let pool = self.pool.clone();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare(
                "SELECT n.id, nm.value as name
                 FROM nodes n
                 JOIN node_metadata nm ON n.id = nm.node_id
                 WHERE n.node_type = 'chapter' AND nm.key = 'name'
                 ORDER BY CAST(SUBSTR(n.id, 9) AS INTEGER)",
            )?;

            let surahs: Vec<(i32, String)> = stmt
                .query_map([], |row| {
                    let id: String = row.get("id")?;
                    // Extract chapter number from "CHAPTER:X" format
                    let chapter_num = id
                        .strip_prefix("CHAPTER:")
                        .and_then(|s| s.parse::<i32>().ok())
                        .ok_or_else(|| {
                            rusqlite::Error::InvalidColumnType(
                                0,
                                "id".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?;
                    let name: String = row.get("name")?;
                    Ok((chapter_num, name))
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(surahs)
        })
        .await?
    }
}
