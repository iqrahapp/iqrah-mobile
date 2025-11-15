/// Migration tool to split old single iqrah.db into content.db + user.db
use rusqlite::{Connection, params};
use anyhow::{Result, Context};
use tracing::{info, warn};

pub struct MigrationStats {
    pub nodes_migrated: usize,
    pub edges_migrated: usize,
    pub arabic_texts_migrated: usize,
    pub translations_migrated: usize,
    pub memory_states_migrated: usize,
    pub propagation_events_migrated: usize,
}

/// Migrate from old single-database to new two-database architecture
pub fn migrate_database(
    old_db_path: &str,
    content_db_path: &str,
    user_db_path: &str,
) -> Result<MigrationStats> {
    info!("Starting database migration from {} to content.db + user.db", old_db_path);

    // Open connections
    let old_conn = Connection::open(old_db_path)
        .context("Failed to open old database")?;

    let content_conn = Connection::open(content_db_path)
        .context("Failed to create content database")?;

    let user_conn = Connection::open(user_db_path)
        .context("Failed to create user database")?;

    // Create schemas
    info!("Creating schemas...");
    let content_schema = include_str!("../migrations/content_schema.sql");
    let user_schema = include_str!("../migrations/user_schema.sql");

    content_conn.execute_batch(content_schema)?;
    user_conn.execute_batch(user_schema)?;

    let mut stats = MigrationStats {
        nodes_migrated: 0,
        edges_migrated: 0,
        arabic_texts_migrated: 0,
        translations_migrated: 0,
        memory_states_migrated: 0,
        propagation_events_migrated: 0,
    };

    // Migrate nodes
    info!("Migrating nodes...");
    {
        let mut stmt = old_conn.prepare("SELECT id, node_type, created_at FROM nodes")?;
        let mut insert_stmt = content_conn.prepare(
            "INSERT INTO nodes (id, node_type, created_at) VALUES (?, ?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        for row in rows {
            let (id, node_type, created_at) = row?;
            insert_stmt.execute(params![id, node_type, created_at])?;
            stats.nodes_migrated += 1;
        }
    }
    info!("Migrated {} nodes", stats.nodes_migrated);

    // Migrate edges
    info!("Migrating edges...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT source_id, target_id, edge_type, distribution_type, param1, param2 FROM edges"
        )?;
        let mut insert_stmt = content_conn.prepare(
            "INSERT INTO edges (source_id, target_id, edge_type, distribution_type, param1, param2)
             VALUES (?, ?, ?, ?, ?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, f64>(5)?,
            ))
        })?;

        for row in rows {
            let (source_id, target_id, edge_type, dist_type, param1, param2) = row?;
            insert_stmt.execute(params![source_id, target_id, edge_type, dist_type, param1, param2])?;
            stats.edges_migrated += 1;
        }
    }
    info!("Migrated {} edges", stats.edges_migrated);

    // Migrate node_metadata -> quran_text (arabic)
    info!("Migrating Arabic text...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT node_id, value FROM node_metadata WHERE key = 'arabic'"
        )?;
        let mut insert_stmt = content_conn.prepare(
            "INSERT INTO quran_text (node_id, arabic) VALUES (?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (node_id, arabic) = row?;
            insert_stmt.execute(params![node_id, arabic])?;
            stats.arabic_texts_migrated += 1;
        }
    }
    info!("Migrated {} Arabic texts", stats.arabic_texts_migrated);

    // Migrate node_metadata -> translations (translation)
    info!("Migrating translations...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT node_id, value FROM node_metadata WHERE key = 'translation'"
        )?;
        let mut insert_stmt = content_conn.prepare(
            "INSERT INTO translations (node_id, language_code, translation) VALUES (?, 'en', ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (node_id, translation) = row?;
            insert_stmt.execute(params![node_id, translation])?;
            stats.translations_migrated += 1;
        }
    }
    info!("Migrated {} translations", stats.translations_migrated);

    // Migrate user_memory_states
    info!("Migrating user memory states...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count
             FROM user_memory_states"
        )?;
        let mut insert_stmt = user_conn.prepare(
            "INSERT INTO user_memory_states
             (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })?;

        for row in rows {
            let (user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count) = row?;
            insert_stmt.execute(params![
                user_id, node_id, stability, difficulty, energy, last_reviewed, due_at, review_count
            ])?;
            stats.memory_states_migrated += 1;
        }
    }
    info!("Migrated {} memory states", stats.memory_states_migrated);

    // Migrate propagation_events
    info!("Migrating propagation events...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT id, source_node_id, event_timestamp FROM propagation_events"
        )?;
        let mut insert_stmt = user_conn.prepare(
            "INSERT INTO propagation_events (id, source_node_id, event_timestamp) VALUES (?, ?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        for row in rows {
            let (id, source_node_id, event_timestamp) = row?;
            insert_stmt.execute(params![id, source_node_id, event_timestamp])?;
            stats.propagation_events_migrated += 1;
        }
    }
    info!("Migrated {} propagation events", stats.propagation_events_migrated);

    // Migrate propagation_details
    info!("Migrating propagation details...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT id, event_id, target_node_id, energy_change, path, reason FROM propagation_details"
        )?;
        let mut insert_stmt = user_conn.prepare(
            "INSERT INTO propagation_details (id, event_id, target_node_id, energy_change, path, reason)
             VALUES (?, ?, ?, ?, ?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;

        for row in rows {
            let (id, event_id, target_node_id, energy_change, path, reason) = row?;
            let reason_text = reason.unwrap_or_else(|| "unknown".to_string());
            insert_stmt.execute(params![id, event_id, target_node_id, energy_change, path, reason_text])?;
        }
    }

    // Migrate session_state
    info!("Migrating session state...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT node_id, session_order FROM session_state"
        )?;
        let mut insert_stmt = user_conn.prepare(
            "INSERT INTO session_state (node_id, session_order) VALUES (?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in rows {
            let (node_id, session_order) = row?;
            insert_stmt.execute(params![node_id, session_order])?;
        }
    }

    // Migrate user_stats
    info!("Migrating user stats...");
    {
        let mut stmt = old_conn.prepare(
            "SELECT key, value FROM user_stats"
        )?;
        let mut insert_stmt = user_conn.prepare(
            "INSERT INTO user_stats (key, value) VALUES (?, ?)"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            insert_stmt.execute(params![key, value])?;
        }
    }

    info!("Migration complete!");
    info!("Summary:");
    info!("  Nodes: {}", stats.nodes_migrated);
    info!("  Edges: {}", stats.edges_migrated);
    info!("  Arabic texts: {}", stats.arabic_texts_migrated);
    info!("  Translations: {}", stats.translations_migrated);
    info!("  Memory states: {}", stats.memory_states_migrated);
    info!("  Propagation events: {}", stats.propagation_events_migrated);

    Ok(stats)
}
