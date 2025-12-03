-- ============================================================================
-- User Database Schema v2.0.0
-- Date: 2024-11-26
-- Consolidated from iterative migrations into single final schema
-- ============================================================================
--
-- This database is READ-WRITE at runtime
-- Contains all user-specific learning progress and state
--
-- Key features:
-- - INTEGER node IDs (i64 encoded) for efficient lookups
-- - FSRS + Energy propagation tracking
-- - Thompson Sampling bandit optimizer state
-- - Session state management
-- ============================================================================

-- ============================================================================
-- SCHEMA VERSION TRACKING
-- ============================================================================
CREATE TABLE schema_version (
    version TEXT NOT NULL PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO schema_version (version, description)
VALUES ('2.0.0', 'User database schema v2 with integer IDs and Thompson Sampling bandit');

-- ============================================================================
-- FSRS + ENERGY TRACKING (Core learning state)
-- ============================================================================
CREATE TABLE user_memory_states (
    user_id TEXT NOT NULL,
    content_key INTEGER NOT NULL,  -- i64 encoded node ID
    stability REAL NOT NULL DEFAULT 0,
    difficulty REAL NOT NULL DEFAULT 0,
    energy REAL NOT NULL DEFAULT 0.0,
    last_reviewed INTEGER NOT NULL DEFAULT 0,   -- epoch milliseconds
    due_at INTEGER NOT NULL DEFAULT 0,          -- epoch milliseconds
    review_count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, content_key)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_ums_user_due ON user_memory_states(user_id, due_at);
CREATE INDEX idx_ums_user_energy ON user_memory_states(user_id, energy);
CREATE INDEX idx_ums_user_last ON user_memory_states(user_id, last_reviewed);

-- ============================================================================
-- SESSION STATE (Ephemeral - for session resume)
-- ============================================================================
CREATE TABLE session_state (
    content_key INTEGER NOT NULL PRIMARY KEY,  -- i64 encoded node ID
    session_order INTEGER NOT NULL
) STRICT, WITHOUT ROWID;

-- ============================================================================
-- ENERGY PROPAGATION TRACKING
-- ============================================================================
CREATE TABLE propagation_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_content_key INTEGER NOT NULL,  -- i64 encoded source node ID
    event_timestamp INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_prop_events_timestamp ON propagation_events(event_timestamp DESC);
CREATE INDEX idx_prop_events_source ON propagation_events(source_content_key);

CREATE TABLE propagation_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id INTEGER NOT NULL,
    target_content_key INTEGER NOT NULL,  -- i64 encoded target node ID
    energy_change REAL NOT NULL,
    path TEXT,
    reason TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES propagation_events(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_prop_details_event ON propagation_details(event_id);
CREATE INDEX idx_prop_details_target ON propagation_details(target_content_key);

-- ============================================================================
-- APP STATE (Settings and statistics)
-- ============================================================================
CREATE TABLE user_stats (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT, WITHOUT ROWID;

-- ============================================================================
-- SCHEDULER V2 BANDIT OPTIMIZER STATE (Thompson Sampling)
-- ============================================================================
-- Stores per-user, per-goal_group bandit arm statistics
-- Each arm corresponds to a UserProfile preset (e.g., "Balanced", "FoundationHeavy")
-- Thompson Sampling uses Beta(successes, failures) to choose arms
CREATE TABLE user_bandit_state (
    user_id TEXT NOT NULL,
    goal_group TEXT NOT NULL,        -- 'memorization', 'vocab', 'tajweed', etc.
    profile_name TEXT NOT NULL,      -- 'Balanced', 'FoundationHeavy', etc.
    successes REAL NOT NULL DEFAULT 1.0,  -- Beta distribution alpha
    failures REAL NOT NULL DEFAULT 1.0,   -- Beta distribution beta
    last_updated INTEGER NOT NULL DEFAULT 0,  -- Epoch milliseconds
    PRIMARY KEY (user_id, goal_group, profile_name)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_bandit_user_group ON user_bandit_state(user_id, goal_group);
CREATE INDEX idx_bandit_updated ON user_bandit_state(last_updated DESC);

-- ============================================================================
-- NOTES
-- ============================================================================
-- Thompson Sampling Algorithm:
-- 1. For each arm (profile), sample from Beta(successes, failures)
-- 2. Choose the arm with the highest sampled value
-- 3. After session completion:
--    - reward = 0.6 * accuracy + 0.4 * completion_rate
--    - successes += reward
--    - failures += (1.0 - reward)
--
-- Profile names match the ProfileName enum in Rust:
-- - 'Balanced': w_urgency=1.0, w_readiness=1.0, w_foundation=1.0, w_influence=1.0
-- - 'FoundationHeavy': w_urgency=0.8, w_readiness=1.0, w_foundation=1.5, w_influence=0.8
-- - 'InfluenceHeavy': w_urgency=0.8, w_readiness=1.0, w_foundation=0.8, w_influence=1.5
-- - 'UrgencyHeavy': w_urgency=1.5, w_readiness=0.8, w_foundation=1.0, w_influence=0.8
--
-- Initial state (1.0, 1.0) represents uninformed prior (uniform distribution)
