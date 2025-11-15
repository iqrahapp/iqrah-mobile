-- Migration v1: Initial user database schema

-- User Memory States (FSRS + Energy)
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,
    due_at INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, node_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);

-- Propagation Events
CREATE TABLE propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_node_id TEXT NOT NULL,
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);

-- Propagation Details
CREATE TABLE propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_node_id TEXT NOT NULL,
    energy_change REAL NOT NULL,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_node_id);

-- Session State (ephemeral - for session resume)
CREATE TABLE session_state (
    node_id TEXT NOT NULL PRIMARY KEY,
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- User Stats
CREATE TABLE user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;
