-- ============================================================================
-- Session tables for persistent session tracking
-- Date: 2025-01-15
-- ============================================================================

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    goal_id TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    items_count INTEGER NOT NULL,
    items_completed INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_sessions_user_started ON sessions(user_id, started_at DESC);
CREATE INDEX idx_sessions_user_completed ON sessions(user_id, completed_at);

CREATE TABLE session_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    node_id INTEGER NOT NULL,
    exercise_type TEXT NOT NULL,
    grade INTEGER NOT NULL,
    duration_ms INTEGER,
    completed_at INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_session_items_session ON session_items(session_id, id);
CREATE INDEX idx_session_items_node ON session_items(node_id);
