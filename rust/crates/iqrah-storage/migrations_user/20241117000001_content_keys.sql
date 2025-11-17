-- User Database Migration: node_id → content_key
-- This migration renames node_id to content_key for clarity and consistency with v2 schema
-- Date: 2025-11-17

-- SQLite doesn't support ALTER COLUMN RENAME directly in all versions
-- We need to recreate the table with the new column name

-- Step 1: Create new table with content_key
CREATE TABLE user_memory_states_new (
    user_id TEXT NOT NULL,
    content_key TEXT NOT NULL,  -- Renamed from node_id
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch milliseconds
    due_at INTEGER NOT NULL DEFAULT 0,          -- epoch milliseconds
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, content_key)
) STRICT, WITHOUT ROWID;

-- Step 2: Copy data from old table
INSERT INTO user_memory_states_new
SELECT
    user_id,
    node_id as content_key,
    stability,
    difficulty,
    energy,
    last_reviewed,
    due_at,
    review_count
FROM user_memory_states;

-- Step 3: Drop old table
DROP TABLE user_memory_states;

-- Step 4: Rename new table to original name
ALTER TABLE user_memory_states_new RENAME TO user_memory_states;

-- Step 5: Recreate indexes with new column name
CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);
CREATE INDEX idx_ums_user_last ON user_memory_states(user_id, last_reviewed);

-- Update session_state table similarly
CREATE TABLE session_state_new (
    content_key TEXT NOT NULL PRIMARY KEY,  -- Renamed from node_id
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

INSERT INTO session_state_new
SELECT node_id as content_key, session_order
FROM session_state;

DROP TABLE session_state;
ALTER TABLE session_state_new RENAME TO session_state;

-- Update propagation_events table
CREATE TABLE propagation_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_content_key TEXT NOT NULL,  -- Renamed from source_node_id
    event_timestamp INTEGER NOT NULL
) STRICT;

INSERT INTO propagation_events_new (id, source_content_key, event_timestamp)
SELECT id, source_node_id, event_timestamp
FROM propagation_events;

DROP TABLE propagation_events;
ALTER TABLE propagation_events_new RENAME TO propagation_events;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);
CREATE INDEX idx_prop_events_source ON propagation_events(source_content_key);

-- Update propagation_details table
CREATE TABLE propagation_details_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_content_key TEXT NOT NULL,  -- Renamed from target_node_id
    energy_change REAL NOT NULL,
    path TEXT,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

INSERT INTO propagation_details_new (id, event_id, target_content_key, energy_change, path, reason)
SELECT id, event_id, target_node_id, energy_change, path, reason
FROM propagation_details;

DROP TABLE propagation_details;
ALTER TABLE propagation_details_new RENAME TO propagation_details;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_content_key);
