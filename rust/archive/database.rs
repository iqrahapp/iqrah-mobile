use anyhow::Result;
use rusqlite::Connection;

/// Create schema with indexes and constraints
pub fn create_schema(conn: &Connection) -> Result<()> {
    // Nodes: Individual learning items (words/phrases)

    conn.execute(
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            node_type TEXT NOT NULL CHECK (node_type IN ('root', 'lemma', 'word', 'word_instance', 'verse', 'chapter', 'knowledge')),
            created_at INTEGER NOT NULL
        )",
        [],
    )?;

    // Node metadata: Arabic text, translations, etc.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS node_metadata (
            node_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (node_id, key),
            FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Edges: Relationships between nodes for propagation
    conn.execute(
        "CREATE TABLE IF NOT EXISTS edges (
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            edge_type INTEGER NOT NULL CHECK (edge_type IN (0, 1)), -- 0:Dependency, 1:Knowledge
            distribution_type INTEGER NOT NULL CHECK (distribution_type IN (0, 1, 2)), -- 0:Const, 1:Normal, 2:Beta
            param1 REAL NOT NULL DEFAULT 0.0,
            param2 REAL NOT NULL DEFAULT 0.0,
            PRIMARY KEY (source_id, target_id),
            FOREIGN KEY (source_id) REFERENCES nodes(id) ON DELETE CASCADE,
            FOREIGN KEY (target_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // User memory states: FSRS scheduling data per user per node
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_memory_states (
            user_id TEXT NOT NULL,
            node_id TEXT NOT NULL,
            stability REAL NOT NULL DEFAULT 0,
            difficulty REAL NOT NULL DEFAULT 0,
            energy REAL NOT NULL DEFAULT 0.0,           -- mastery 0-1 scale
            last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch ms
            due_at INTEGER NOT NULL DEFAULT 0,          -- epoch ms
            review_count INTEGER NOT NULL DEFAULT 0,
            priority_score REAL NOT NULL DEFAULT 0.0,    -- for scheduling
            PRIMARY KEY (user_id, node_id),
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS propagation_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_node_id TEXT NOT NULL,
            event_timestamp INTEGER NOT NULL,
            FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS propagation_details (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id INTEGER NOT NULL,
            target_node_id TEXT NOT NULL,
            energy_change REAL NOT NULL,
            path TEXT,
            reason TEXT,
            FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE,
            FOREIGN KEY (target_node_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // High-impact indexes for due items queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ums_user_due ON user_memory_states(user_id, due_at)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ums_user_last ON user_memory_states(user_id, last_reviewed)",
        [],
    )?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_ums_priority ON user_memory_states(user_id, priority_score)", [])?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_source ON edges(source_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_target ON edges(target_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_propagation_events_timestamp ON propagation_events(event_timestamp DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_propagation_details_event ON propagation_details(event_id)",
        [],
    )?;

    // Session persistence: stores current session items
    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_state (
            node_id TEXT NOT NULL PRIMARY KEY,
            session_order INTEGER NOT NULL,
            FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // User statistics: daily counts and streaks
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_stats (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}
