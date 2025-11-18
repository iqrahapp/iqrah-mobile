-- Scheduler v2.1 Bandit State
-- Stores Thompson Sampling bandit state for hyper-personalized scheduling
-- Date: 2025-11-17

-- ============================================================================
-- BANDIT OPTIMIZER STATE (Thompson Sampling)
-- ============================================================================

-- Stores per-user, per-goal_group bandit arm statistics
-- Each arm corresponds to a UserProfile preset (e.g., "Balanced", "FoundationHeavy")
-- Thompson Sampling uses Beta(successes, failures) to choose arms
CREATE TABLE IF NOT EXISTS user_bandit_state (
    user_id TEXT NOT NULL,
    goal_group TEXT NOT NULL,        -- 'memorization', 'vocab', 'tajweed', etc.
    profile_name TEXT NOT NULL,      -- 'Balanced', 'FoundationHeavy', 'InfluenceHeavy', 'UrgencyHeavy'
    successes REAL NOT NULL DEFAULT 1.0,  -- Beta distribution alpha parameter
    failures REAL NOT NULL DEFAULT 1.0,   -- Beta distribution beta parameter
    last_updated INTEGER NOT NULL DEFAULT 0,  -- Epoch milliseconds
    PRIMARY KEY (user_id, goal_group, profile_name)
) STRICT, WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_bandit_user_group ON user_bandit_state(user_id, goal_group);
CREATE INDEX IF NOT EXISTS idx_bandit_updated ON user_bandit_state(last_updated DESC);

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

-- Profile names should match the ProfileName enum in Rust:
-- - 'Balanced': w_urgency=1.0, w_readiness=1.0, w_foundation=1.0, w_influence=1.0
-- - 'FoundationHeavy': w_urgency=0.8, w_readiness=1.0, w_foundation=1.5, w_influence=0.8
-- - 'InfluenceHeavy': w_urgency=0.8, w_readiness=1.0, w_foundation=0.8, w_influence=1.5
-- - 'UrgencyHeavy': w_urgency=1.5, w_readiness=0.8, w_foundation=1.0, w_influence=0.8

-- Initial state (1.0, 1.0) represents uninformed prior (uniform distribution)
-- This ensures fair exploration in the beginning
