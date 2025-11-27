-- User Database Migration: Convert content_key from TEXT to INTEGER
-- This migration converts content_key (node IDs) from string format to i64 integers
-- Date: 2025-11-26

-- SQLite doesn't support ALTER COLUMN type changes
-- We need to recreate tables with the new column type

-- Step 1: Create new user_memory_states table with INTEGER content_key
CREATE TABLE user_memory_states_new (
    user_id TEXT NOT NULL,
    content_key INTEGER NOT NULL,  -- Changed from TEXT to INTEGER
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch milliseconds
    due_at INTEGER NOT NULL DEFAULT 0,          -- epoch milliseconds
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, content_key)
) STRICT, WITHOUT ROWID;

-- Step 2: Copy data from old table (no data to migrate in new databases)
-- Old databases would need a conversion function from TEXT to INTEGER node_ids

-- Step 3: Drop old table
DROP TABLE user_memory_states;

-- Step 4: Rename new table to original name
ALTER TABLE user_memory_states_new RENAME TO user_memory_states;

-- Step 5: Recreate indexes
CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);
CREATE INDEX idx_ums_user_last ON user_memory_states(user_id, last_reviewed);

-- Update session_state table
CREATE TABLE session_state_new (
    content_key INTEGER NOT NULL PRIMARY KEY,  -- Changed from TEXT to INTEGER
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

DROP TABLE session_state;
ALTER TABLE session_state_new RENAME TO session_state;

-- Update propagation_events table
CREATE TABLE propagation_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_content_key INTEGER NOT NULL,  -- Changed from TEXT to INTEGER
    event_timestamp INTEGER NOT NULL
) STRICT;

DROP TABLE propagation_events;
ALTER TABLE propagation_events_new RENAME TO propagation_events;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);
CREATE INDEX idx_prop_events_source ON propagation_events(source_content_key);

-- Update propagation_details table
CREATE TABLE propagation_details_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_content_key INTEGER NOT NULL,  -- Changed from TEXT to INTEGER
    energy_change REAL NOT NULL,
    path TEXT,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

DROP TABLE propagation_details;
ALTER TABLE propagation_details_new RENAME TO propagation_details;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_content_key);
