-- User Database Schema
-- This database is READ-WRITE at runtime
-- Contains all user-specific learning progress and state

-- User Memory States (FSRS + Energy)
CREATE TABLE IF NOT EXISTS user_memory_states (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch milliseconds
    due_at INTEGER NOT NULL DEFAULT 0,          -- epoch milliseconds
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, node_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX IF NOT EXISTS idx_ums_user_energy ON user_memory_states(user_id, energy);
CREATE INDEX IF NOT EXISTS idx_ums_user_last ON user_memory_states(user_id, last_reviewed);

-- Propagation Events
CREATE TABLE IF NOT EXISTS propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_node_id TEXT NOT NULL,
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_prop_events_source ON propagation_events(source_node_id);

-- Propagation Details
CREATE TABLE IF NOT EXISTS propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_node_id TEXT NOT NULL,
    energy_change REAL NOT NULL,
    path TEXT,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX IF NOT EXISTS idx_prop_details_target ON propagation_details(target_node_id);

-- Session State (ephemeral - for session resume)
CREATE TABLE IF NOT EXISTS session_state (
    node_id TEXT NOT NULL PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- User Stats (daily counts, streaks, etc.)
CREATE TABLE IF NOT EXISTS user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- App Settings (new in Sprint 7)
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
